use bevy::{input::common_conditions::{input_just_pressed, input_just_released}, prelude::*, render::primitives::Aabb, window::PrimaryWindow};

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
		TransformBundle {
			local: Transform {
				translation: Vec3::new(0.0, 0.0, 500.0),
				..Default::default()
			},
			..Default::default()
		},
	))
		.with_children(|parent| {
			parent.spawn((
				DragAttach,
				TransformBundle::default(),
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
		z: 0.0,
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
	let (drag_attach, mut drag_attach_transform) = drag_attach.single_mut();
	
	for (entity, parent, transform, aabb) in draggable.iter() {
		let relative_translation = cursor_transform.translation.truncate() - transform.translation().truncate();
		if relative_translation.x < -aabb.half_extents.x
			|| relative_translation.y < -aabb.half_extents.y
			|| relative_translation.x > aabb.half_extents.x
			|| relative_translation.y > aabb.half_extents.y
		{
			continue;
		}
		
		drag_attach_transform.translation = -relative_translation.extend(0.0);
		
		commands.entity(entity).insert(Dragging {
			previous_parent: parent.get(),
		});
		
		commands.entity(drag_attach).push_children(&[entity]);
	}
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
