use std::{
    os::windows::thread,
    sync::{atomic::AtomicI32, Arc, Mutex},
};

use rand::prelude::*;
use rayon::prelude::*;
#[derive(Clone, Copy, PartialEq, Eq)]
struct Animal {
    litter_size: u8,
    aggressiveness: u8,
    max_food: u16,
    food_eaten: i32,
}
impl Animal {
    pub fn mutate(self, thread_rng: &mut ThreadRng) -> Self {
        Self {
            litter_size: (self.litter_size as i16 + thread_rng.gen_range(-3..3)) as u8,
            aggressiveness: (self.aggressiveness as i16 + thread_rng.gen_range(-3..3)) as u8,
            max_food: (self.max_food as i32 + thread_rng.gen_range(-5..5)) as u16,

            food_eaten: 0,
        }
    }
}
fn main() {
    let mut animals: Vec<Animal> = vec![];
    let mut thread_rng = thread_rng();
    for _ in 0..30 {
        animals.push(Animal {
            litter_size: thread_rng.gen_range(0..5),
            aggressiveness: 6,
            max_food: thread_rng.gen_range(10..20),
            food_eaten: 0,
        })
    }
    let death_minimum = 200;
    let total_food = Arc::new(Mutex::new(1000));
    let mutation_rate = 0.005;
    let aggression_loss = 0.4;
    for gen in 0..100000 {
        if animals.len() == 0 {
            panic!("eee");
        }
        let total_aggression: f64 = animals.iter().map(|x| x.aggressiveness as f64).sum();
        let food_per_aggression: f64 = { *total_food.lock().unwrap() as f64 / total_aggression };
        animals.par_iter_mut().for_each(|animal| {
            if animal.food_eaten > animal.max_food as i32 {
                panic!("{}, {}", animal.food_eaten, animal.max_food);
            } else {
                let mut eat_amount = ((animal.aggressiveness as f64 * food_per_aggression) as i32)
                    .min((animal.aggressiveness as f64 * (aggression_loss + 1.0)) as i32)
                    .min(animal.max_food as i32 - animal.food_eaten);
                eat_amount = eat_amount.clamp(0, eat_amount);
                {
                    *total_food.lock().unwrap() -= eat_amount;
                }
                animal.food_eaten += eat_amount;
            }

            assert!(*total_food.lock().unwrap() > 0);
            animal.food_eaten -= (animal.aggressiveness as f64 * aggression_loss) as i32;
            // println!("{}", animal.food_eaten);
        });
        assert!(*total_food.lock().unwrap() > 0);

        let mut to_add = vec![];
        for animal in &mut animals {
            if animal.food_eaten >= animal.litter_size as i32 * 4 {
                to_add.push(*animal);
                animal.food_eaten -= animal.litter_size as i32 * 4;
            }
        }
        animals.retain(|x| x.food_eaten >= death_minimum);

        for animal in to_add {
            for _ in 0..animal.litter_size {
                if thread_rng.gen_bool(mutation_rate) {
                    animals.push(animal.mutate(&mut thread_rng));
                }
                animals.push(animal);
            }
        }
        // println!(
        //     "gen : {gen} , animals: {}, total aggression: {}, food per aggression {}",
        //     animals.len(),
        //     total_aggression,
        //     food_per_aggression
        // );
        println!(
            "{}, {}, {}, {}, {}, {}",
            gen,
            animals.len(),
            animals.iter().map(|x| x.aggressiveness as f64).sum::<f64>() / animals.len() as f64,
            animals.iter().map(|x| x.litter_size as f64).sum::<f64>() / animals.len() as f64,
            animals.iter().map(|x| x.food_eaten as f64).sum::<f64>() / animals.len() as f64,
            *total_food.lock().unwrap()
        );
        {
            let mut total_food = total_food.lock().unwrap();
            *total_food += (0.90_f64).powf(*total_food as f64) as i32;
        }
    }
}
