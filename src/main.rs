#![forbid(unsafe_code)]
#![deny(non_snake_case)]

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_svg::prelude::*;
use card::*;
use strum::IntoEnumIterator;

mod card;

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
		.add_systems(Startup, (spawn_camera, spawn_cards))
		.run();
}

fn spawn_camera(mut commands: Commands) {
	let mut camera = Camera2dBundle::default();
	camera.projection.scaling_mode = ScalingMode::AutoMin {
		min_width: 1300.0,
		min_height: 760.0,
	};
	
	commands.spawn(camera);
}

fn spawn_cards(mut commands: Commands, card_assets: Res<CardAssets>) {
	let scale = Vec3::new(0.5, 0.5, 0.5);
	
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
			x: x as f32 * 100.0 - 6.0 * 100.0,
			y: y as f32 * 190.0 - 1.5 * 190.0,
			z: 0.0,
		};
		
		commands.spawn(Svg2dBundle {
			svg,
			transform: Transform {
				translation,
				scale,
				..Default::default()
			},
			..Default::default()
		});
	}
}
