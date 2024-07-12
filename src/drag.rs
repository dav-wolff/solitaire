use bevy::{ecs::query::QuerySingleError, input::common_conditions::{input_just_pressed, input_just_released}, prelude::*, render::primitives::Aabb, utils::FloatOrd, window::PrimaryWindow};

const CURSOR_Z: f32 = 500.0;

#[derive(Component, Debug)]
pub struct Draggable(pub bool);

#[derive(Component, Debug)]
pub struct DropTarget;

#[derive(Event, Debug)]
pub struct DropEvent {
	dropped: Entity,
	target: Entity,
	previous_parent: Entity,
}

impl DropEvent {
	pub fn dropped(&self) -> Entity {
		self.dropped
	}
	
	pub fn target(&self) -> Entity {
		self.target
	}
	
	pub fn previous_parent(&self) -> Entity {
		self.previous_parent
	}
	
	pub fn attach_to_target(&self, commands: &mut Commands) {
		commands.entity(self.target).push_children(&[self.dropped]);
	}
	
	pub fn return_to_parent(&self, commands: &mut Commands) {
		commands.entity(self.previous_parent).push_children(&[self.dropped]);
	}
}

#[derive(Debug)]
pub struct DragPlugin;

impl Plugin for DragPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<DropEvent>()
			.add_systems(Startup, setup)
			.add_systems(PreUpdate, (
				update_cursor,
				drag.run_if(input_just_pressed(MouseButton::Left)),
				drop.run_if(input_just_released(MouseButton::Left)),
			).chain());
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
	draggables: Query<(Entity, &Draggable, &Parent, &Transform, &GlobalTransform, &Aabb), (Without<Cursor>, Without<DragAttach>)>,
) {
	let cursor_transform = cursor.single();
	let cursor_translation = cursor_transform.translation;
	let (drag_attach, mut drag_attach_transform) = drag_attach.single_mut();
	
	let Some((entity, Draggable(draggable), parent, transform, relative_translation, _)) = draggables.iter()
		.map(|(entity, draggable, parent, transform, global_transform, aabb)| {
			let relative_translation = cursor_translation - global_transform.translation();
			(entity, draggable, parent, transform, relative_translation, aabb)
		})
		.filter(|(_, _, _, _, relative_translation, aabb)| inside_bounding_box(relative_translation.truncate(), **aabb))
		.max_by_key(|(_, _, _, _, relative_translation, _)| FloatOrd(-relative_translation.z))
	else {
		return;
	};
	
	if !draggable {
		return;
	}
	
	drag_attach_transform.translation = -relative_translation - transform.translation;
	drag_attach_transform.translation.z = 0.0;
	
	commands.entity(entity).insert(Dragging {
		previous_parent: parent.get(),
	});
	
	commands.entity(drag_attach).push_children(&[entity]);
}

fn drop(
	mut commands: Commands,
	mut event_writer: EventWriter<DropEvent>,
	cursor: Query<&Transform, With<Cursor>>,
	dragging: Query<(Entity, &Dragging)>,
	drop_targets: Query<(Entity, &GlobalTransform, &Aabb), (With<DropTarget>, Without<Dragging>)>,
) {
	let (dropped, Dragging {previous_parent}) = match dragging.get_single() {
		Ok(dragging) => dragging,
		Err(QuerySingleError::NoEntities(_)) => return,
		Err(QuerySingleError::MultipleEntities(_)) => panic!("There should be at most one Dragging"),
	};
	
	let previous_parent = *previous_parent;
	let cursor_translation = cursor.single().translation.truncate();
	
	commands.entity(dropped).remove::<Dragging>();
	
	let Some((target, _, _)) = drop_targets.iter()
		.filter(|(_, transform, aabb)| {
			let relative_translation = cursor_translation - transform.translation().truncate();
			inside_bounding_box(relative_translation, **aabb)
		})
		.max_by_key(|(_, transform, _)| FloatOrd(transform.translation().z))
	else {
		commands.entity(previous_parent).push_children(&[dropped]);
		return;
	};
	
	event_writer.send(DropEvent {
		dropped,
		target,
		previous_parent,
	});
}

fn inside_bounding_box(position: Vec2, aabb: Aabb) -> bool {
	position.x > -aabb.half_extents.x
		&& position.y > -aabb.half_extents.y
		&& position.x < aabb.half_extents.x
		&& position.y < aabb.half_extents.y
}
