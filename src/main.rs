#![forbid(unsafe_code)]
#![deny(non_snake_case)]
#![allow(clippy::type_complexity)]

use std::{cmp::min, collections::VecDeque, ops::DerefMut};

use bevy::{asset::AssetMetaCheck, prelude::*, render::camera::ScalingMode};
use bevy_svg::prelude::*;
use card::*;
use deck::Deck;
use drag::{DragPlugin, Draggable, DropEvent, DropTarget};

mod card;
mod deck;
mod drag;

fn main() {
	#[cfg(feature = "native")]
	let window = Window {
		title: "Solitaire".into(),
		resolution: (1300.0, 760.0).into(),
		..Default::default()
	};
	
	#[cfg(feature = "web")]
	let window = Window {
		title: "Solitaire".into(),
		canvas: option_env!("SOLITAIRE_CANVAS_ID").map(Into::into),
		prevent_default_event_handling: false,
		..Default::default()
	};
	
	App::new()
		.insert_resource(AssetMetaCheck::Never)
		.add_plugins(DefaultPlugins.set(WindowPlugin {
			primary_window: Some(window),
			..Default::default()
		}))
		.add_plugins(bevy_svg::prelude::SvgPlugin)
		.add_plugins(CardAssetsPlugin(env!("SOLITAIRE_CARDS_LOCATION").into()))
		.add_plugins(DragPlugin)
		.add_systems(Startup, (spawn_camera, spawn_cards))
		.add_systems(PreUpdate, update_stack_children)
		.add_systems(Update, handle_dropped_card)
		.add_systems(Update, (
			resize_stack,
			make_draggable,
		))
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

#[derive(Component, Default, Debug)]
struct Slot {
	stack: Vec<Entity>,
}

fn spawn_cards(mut commands: Commands, card_assets: Res<CardAssets>) {
	let slot_svg = card_assets.slot();
	
	let mut deck = Deck::shuffled();
	
	for x in 0..10 {
		let cards_in_slot = min(10 - x, 8);
		
		let mut parent = commands.spawn((
			Svg2dBundle {
				transform: Transform {
					translation: (x as f32 * 250.0 - 4.5 * 250.0, 400.0, 0.0).into(),
					..Default::default()
				},
				svg: slot_svg.clone(),
				..Default::default()
			},
			DropTarget,
			Slot::default(),
		)).id();
		
		for _ in 0..cards_in_slot {
			let card = deck.draw().expect("This layout shouldn't require more cards than in the deck");
			let card_svg = card_assets.get(card);
			
			let child = commands.spawn((
				Svg2dBundle {
					transform: Transform {
						translation: (0.0, 0.0, 1.0).into(),
						..Default::default()
					},
					svg: card_svg.clone(),
					..Default::default()
				},
				card,
				Draggable(false),
				DropTarget,
			)).id();
			
			commands.entity(parent).push_children(&[child]);
			parent = child;
		}
	}
}

fn handle_dropped_card(
	mut commands: Commands,
	mut event_reader: EventReader<DropEvent>,
	cards: Query<&Card>,
	unoccupied_cards: Query<&Card, Without<Children>>,
	slots: Query<(), With<Slot>>,
	mut parents: Query<&mut Parent, With<Card>>,
) {
	for event in event_reader.read() {
		let Ok(dropped) = cards.get(event.dropped()) else {
			continue;
		};
		
		let mut attach_to_target = || {
			event.attach_to_target(&mut commands);
			
			if let Ok(mut parent) = parents.get_mut(event.previous_parent()) {
				// trigger change detection to get previous stack to update
				parent.deref_mut();
			}
		};
		
		if slots.get(event.target()).is_ok() {
			attach_to_target();
			continue;
		}
		
		if let Ok(target) = unoccupied_cards.get(event.target()) {
			if target.value.as_number() != dropped.value.as_number() + 1 {
				event.return_to_parent(&mut commands);
				continue;
			}
			
			attach_to_target();
			event.attach_to_target(&mut commands);
			continue;
		}
		
		event.return_to_parent(&mut commands);
	}
}

fn update_stack_children(
	moved_cards: Query<Entity, (With<Card>, Changed<Parent>)>,
	cards: Query<(&Parent, Option<&Children>)>,
	mut slots: Query<&mut Slot>,
) {
	for moved_card in moved_cards.iter() {
		let mut cards_in_stack = VecDeque::new();
		
		// Add ancestors
		let mut current = moved_card;
		while let Ok((parent, _)) = cards.get(current) {
			cards_in_stack.push_front(current);
			current = parent.get();
		}
		
		let slot = slots.get_mut(current);
		
		// Add descendents
		let mut current = moved_card;
		while let Ok((_, Some(children))) = cards.get(current) {
			assert_eq!(children.len(), 1, "Cards shouldn't have multiple children");
			// Doesn't check whether the child is actually a card, probably not necessary
			cards_in_stack.push_back(children[0]);
			current = children[0];
		}
		
		if let Ok(mut slot) = slot {
			slot.stack = cards_in_stack.into();
		}
	}
}

fn resize_stack(
	changed_slots: Query<&Slot, Changed<Slot>>,
	mut cards: Query<&mut Transform, With<Card>>,
) {
	for changed_slot in changed_slots.iter() {
		let stack_size = changed_slot.stack.len();
		let distance = (1000.0 / stack_size as f32).clamp(0.0, 100.0);
		
		if let Some(&first) = changed_slot.stack.first() {
			let mut transform = cards.get_mut(first).expect("Slot::stack should only contain cards");
			transform.translation = Vec3::new(0.0, 0.0, 1.0);
		}
		
		for &card in changed_slot.stack.iter().skip(1) {
			let mut transform = cards.get_mut(card).expect("Slot::stack should only contain cards");
			transform.translation = Vec3::new(0.0, -distance, 1.0);
		}
	}
}

fn make_draggable(
	changed_slots: Query<&Slot, Changed<Slot>>,
	mut cards: Query<(&mut Draggable, &Card)>,
) {
	for changed_slot in changed_slots.iter() {
		let Some(&top) = changed_slot.stack.last() else {
			continue;
		};
		
		let (mut draggable, &(mut prev_card)) = cards.get_mut(top).expect("Slot::stack should only contain cards");
		draggable.0 = true;
		
		let mut is_draggable = true;
		for &card in changed_slot.stack.iter().rev().skip(1) {
			let (mut draggable, &card) = cards.get_mut(card).expect("Slot::stack should only contain cards");
			
			if dbg!(is_draggable) {
				is_draggable = prev_card.suit == card.suit && prev_card.value.as_number() == card.value.as_number() - 1;
			}
			
			draggable.0 = is_draggable;
			prev_card = card;
		}
	}
}
