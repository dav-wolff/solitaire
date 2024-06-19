#![forbid(unsafe_code)]
#![deny(non_snake_case)]
#![allow(clippy::type_complexity)]

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_svg::prelude::*;
use card::*;
use drag::{DragPlugin, Draggable};
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
		let svg = card_assets.get(Card {
			suit,
			value,
		});
		
		let translation = Vec3 {
			x: x as f32 * 200.0 - 6.0 * 200.0,
			y: y as f32 * 250.0 - 1.5 * 380.0,
			z: y as f32,
		};
		
		commands.spawn(SpatialBundle {
			transform: Transform {
				translation,
				..Default::default()
			},
			..Default::default()
		})
			.with_children(|parent| {
				parent.spawn((
					Svg2dBundle {
						svg,
						..Default::default()
					},
					Draggable,
				));
			});
	}
}
