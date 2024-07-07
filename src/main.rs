#![forbid(unsafe_code)]
#![deny(non_snake_case)]
#![allow(clippy::type_complexity)]

use std::collections::VecDeque;

use bevy::{prelude::*, render::camera::ScalingMode};
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
		.add_systems(Update, resize_stack)
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

#[derive(Component, Debug)]
struct Slot;

fn spawn_cards(mut commands: Commands, card_assets: Res<CardAssets>) {
	let slot_svg = card_assets.slot();
	
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
		
		let card_svg = card_assets.get(card);
		
		let translation = Vec3 {
			x: x as f32 * 200.0 - 6.0 * 200.0,
			y: y as f32 * 380.0 - 1.5 * 380.0,
			z: 0.0,
		};
		
		commands.spawn((
			Svg2dBundle {
				svg: slot_svg.clone(),
				transform: Transform {
					translation,
					..Default::default()
				},
				..Default::default()
			},
			DropTarget,
			Slot,
		))
			.with_children(|parent| {
				parent.spawn((
					Svg2dBundle {
						svg: card_svg,
						transform: Transform {
							translation: Vec3::new(0.0, 0.0, 1.0),
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
	slots: Query<(), With<Slot>>,
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
		
		if slots.get(event.target()).is_ok() {
			event.attach_to_target(&mut commands);
			continue;
		}
		
		event.return_to_parent(&mut commands);
	}
}

fn resize_stack(
	dropped_cards: Query<Entity, (With<Card>, Changed<Parent>)>,
	mut cards: Query<(&mut Transform, &Parent, Option<&Children>), With<Card>>,
	slots: Query<(), With<Slot>>,
) {
	for dropped_card in dropped_cards.iter() {
		let mut cards_in_pile = VecDeque::new();
		
		// Add ancestors
		let mut current = dropped_card;
		while let Ok((_, parent, _)) = cards.get(current) {
			cards_in_pile.push_front(current);
			current = parent.get();
		}
		
		let top = current;
		
		// Add descendents
		let mut current = dropped_card;
		while let Ok((_, _, Some(children))) = cards.get(current) {
			assert_eq!(children.len(), 1, "Cards shouldn't have multiple children");
			// Doesn't check whether the child is actually a card, probably not necessary
			cards_in_pile.push_back(children[0]);
			current = children[0];
		}
		
		for card in cards_in_pile.iter().skip(1) {
			let (mut transform, _, _) = cards.get_mut(*card).expect("The entity originates from the same query");
			transform.translation = Vec3::new(0.0, -50.0, 1.0);
		}
		
		if slots.get(top).is_ok() {
			let last_card = cards_in_pile.front()
				.expect("At least one card must exist, as this function was called for it (unless it doesn't have a Transform)");
			let (mut transform, _, _) = cards.get_mut(*last_card).expect("The entity originates from the same query");
			transform.translation = Vec3::new(0.0, 0.0, 1.0);
		}
	}
}
