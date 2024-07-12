use rand::seq::SliceRandom;
use crate::Card;

pub struct Deck {
	cards: Vec<Card>,
	first_available: usize,
}

impl Deck {
	pub fn shuffled() -> Self {
		let mut rng = rand::thread_rng();
		let mut cards: Vec<_> = Card::deck().collect();
		
		cards.shuffle(&mut rng);
		
		Self {
			cards,
			first_available: 0,
		}
	}
	
	pub fn draw(&mut self) -> Option<Card> {
		let card = *self.cards.get(self.first_available)?;
		self.first_available += 1;
		Some(card)
	}
}
