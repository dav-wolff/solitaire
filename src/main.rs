#![forbid(unsafe_code)]
#![deny(non_snake_case)]
#![allow(clippy::type_complexity)]

use bevy::{prelude::*, render::{camera::ScalingMode, primitives::Aabb}};
use bevy_svg::prelude::*;
use card::*;
use drag::{DragPlugin, Draggable, DropEvent, DropTarget};
use strum::IntoEnumIterator;

mod card;
mod drag;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins.set(WindowPlugin {
			primary_window: Some(Window {
				title: "Solitaire".into(),
				resolution: (1300.0, 760.0).into(),
				..Default::default()
			}),
			..Default::default()
		}))
		.add_plugins(bevy_svg::prelude::SvgPlugin)
		.add_plugins(CardAssetsPlugin(env!("SOLITAIRE_CARDS_LOCATION").into()))
		.add_plugins(DragPlugin)
		.add_systems(Startup, (spawn_camera, spawn_cards))
		.add_systems(Update, handle_dropped_card)
		.run();
}

fn spawn_camera(mut commands: Commands) {
	let mut camera = Camera2dBundle::default();
	camera.projection.scaling_mode = ScalingMode::AutoMin {
		min_width: 2600.0,
		min_height: 1520.0,
	};
	
	commands.spawn(camera);
}

fn spawn_cards(mut commands: Commands, card_assets: Res<CardAssets>) {
	for (x, y, suit, value) in Suit::iter()
		.enumerate()
		.flat_map(|(y, suit)| {
			Value::iter()
				.enumerate()
				.map(move |(x, value)| (x, y, suit, value))
		})
	{
		let card = Card {
			suit,
			value,
		};
		
		let svg = card_assets.get(card);
		
		let translation = Vec3 {
			x: x as f32 * 200.0 - 6.0 * 200.0,
			y: y as f32 * 380.0 - 1.5 * 380.0 + 40.0,
			z: 0.0,
		};
		
		commands.spawn((
			SpatialBundle {
				transform: Transform {
					translation,
					..Default::default()
				},
				..Default::default()
			},
			Aabb {
				half_extents: bevy::math::Vec3A::new(180.0, 360.0, 0.0),
				..Default::default()
			},
			DropTarget,
		))
			.with_children(|parent| {
				parent.spawn((
					Svg2dBundle {
						svg,
						transform: Transform {
							translation: Vec3::new(0.0, -50.0, 1.0),
							..Default::default()
						},
						..Default::default()
					},
					Draggable,
					DropTarget,
					card,
				));
			});
	}
}

fn handle_dropped_card(
	mut commands: Commands,
	mut event_reader: EventReader<DropEvent>,
	cards: Query<&Card>,
	unoccupied_cards: Query<&Card, Without<Children>>,
) {
	for event in event_reader.read() {
		let Ok(dropped) = cards.get(event.dropped()) else {
			continue;
		};
		
		if let Ok(target) = unoccupied_cards.get(event.target()) {
			if dropped.suit == target.suit {
				event.attach_to_target(&mut commands);
				continue;
			}
		}
		
		event.return_to_parent(&mut commands);
	}
}
