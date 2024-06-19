use bevy::{input::common_conditions::{input_just_pressed, input_just_released}, prelude::*, render::primitives::Aabb, utils::FloatOrd, window::PrimaryWindow};

const CURSOR_Z: f32 = 500.0;

#[derive(Component, Debug)]
pub struct Draggable;

#[derive(Debug)]
pub struct DragPlugin;

impl Plugin for DragPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup)
			.add_systems(PreUpdate, update_cursor)
			.add_systems(Update, (
				drag.run_if(input_just_pressed(MouseButton::Left)),
				drop.run_if(input_just_released(MouseButton::Left)),
			));
	}
}

#[derive(Component, Debug)]
struct Cursor;

#[derive(Component, Debug)]
struct DragAttach;

fn setup(mut commands: Commands) {
	commands.spawn((
		Cursor,
		SpatialBundle {
			transform: Transform {
				translation: Vec3::new(0.0, 0.0, CURSOR_Z),
				..Default::default()
			},
			..Default::default()
		},
	))
		.with_children(|parent| {
			parent.spawn((
				DragAttach,
				SpatialBundle::default(),
			));
		});
}

fn update_cursor(
	window: Query<&Window, With<PrimaryWindow>>,
	camera: Query<(&Camera, &GlobalTransform)>,
	mut cursor: Query<&mut Transform, With<Cursor>>,
) {
	let window = window.single();
	let (camera, camera_transform) = camera.single();
	let mut cursor_transform = cursor.single_mut();
	
	let Some(position) = window.cursor_position()
		.and_then(|position| camera.viewport_to_world(camera_transform, position))
		.map(|ray| ray.origin.truncate())
	else {
		return;
	};
	
	cursor_transform.translation = Vec3 {
		x: position.x,
		y: position.y,
		z: CURSOR_Z,
	};
}

#[derive(Component, Debug)]
struct Dragging {
	previous_parent: Entity,
}

fn drag(
	mut commands: Commands,
	cursor: Query<&Transform, With<Cursor>>,
	mut drag_attach: Query<(Entity, &mut Transform), (With<DragAttach>, Without<Cursor>)>,
	draggable: Query<(Entity, &Parent, &GlobalTransform, &Aabb), With<Draggable>>,
) {
	let cursor_transform = cursor.single();
	let cursor_translation = cursor_transform.translation;
	let (drag_attach, mut drag_attach_transform) = drag_attach.single_mut();
	
	let Some((entity, parent, relative_translation, _)) = draggable.iter()
		.map(|(entity, parent, transform, aabb)| {
			let relative_translation = cursor_translation - transform.translation();
			(entity, parent, dbg!(relative_translation), aabb)
		})
		.filter(|(_, _, relative_translation, aabb)| {
			relative_translation.x > -aabb.half_extents.x
				&& relative_translation.y > -aabb.half_extents.y
				&& relative_translation.x < aabb.half_extents.x
				&& relative_translation.y < aabb.half_extents.y
		})
		.max_by_key(|(_, _, relative_translation, _)| FloatOrd(-relative_translation.z))
	else {
		return;
	};
	
	drag_attach_transform.translation = -relative_translation;
	drag_attach_transform.translation.z = 0.0;
	
	commands.entity(entity).insert(Dragging {
		previous_parent: parent.get(),
	});
	
	commands.entity(drag_attach).push_children(&[entity]);
}

fn drop(
	mut commands: Commands,
	dragging: Query<(Entity, &Dragging)>,
) {
	for (entity, Dragging {previous_parent}) in dragging.iter() {
		commands.entity(*previous_parent).push_children(&[entity]);
		commands.entity(entity).remove::<Dragging>();
	}
}
