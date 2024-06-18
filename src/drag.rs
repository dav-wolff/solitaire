use bevy::{input::common_conditions::{input_just_pressed, input_just_released}, prelude::*, render::primitives::Aabb, window::PrimaryWindow};

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
	));
}

#[derive(Component, Debug)]
struct Cursor;

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
pub struct Draggable;

#[derive(Component, Debug)]
pub struct Dragging {
	previous_parent: Entity,
}

fn drag(
	mut commands: Commands,
	cursor: Query<(Entity, &Transform), With<Cursor>>,
	draggable: Query<(Entity, &Parent, &GlobalTransform, &Aabb), With<Draggable>>,
) {
	let (cursor, cursor_transform) = cursor.single();
	
	for (entity, parent, transform, aabb) in draggable.iter() {
		let relative_transform = cursor_transform.translation.truncate() - transform.translation().truncate();
		if relative_transform.x < -aabb.half_extents.x
			|| relative_transform.y < -aabb.half_extents.y
			|| relative_transform.x > aabb.half_extents.x
			|| relative_transform.y > aabb.half_extents.y
		{
			continue;
		}
		
		commands.entity(entity).insert(Dragging {
			previous_parent: parent.get(),
		});
		
		commands.entity(cursor).push_children(&[entity]);
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
