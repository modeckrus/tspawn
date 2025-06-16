//! Complex data structures example.
//!
//! This example shows how to use the tspawn crate with more complex
//! data structures like hashmaps, custom structs, and nested data.

use std::collections::HashMap;
use tspawn::{tspawn, A};

#[derive(Debug, Clone)]
struct User {
    id: u32,
    name: String,
    score: i32,
}

impl User {
    fn new(id: u32, name: &str, score: i32) -> Self {
        User {
            id,
            name: name.to_string(),
            score,
        }
    }
}

#[derive(Debug, Clone)]
struct GameState {
    users: HashMap<u32, User>,
    current_round: u32,
    game_active: bool,
}

impl GameState {
    fn new() -> Self {
        GameState {
            users: HashMap::new(),
            current_round: 1,
            game_active: true,
        }
    }

    fn add_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }

    fn update_user_score(&mut self, user_id: u32, score_delta: i32) {
        if let Some(user) = self.users.get_mut(&user_id) {
            user.score += score_delta;
        }
    }

    fn get_leaderboard(&self) -> Vec<(&User, i32)> {
        let mut users: Vec<_> = self.users.values().map(|u| (u, u.score)).collect();
        users.sort_by(|a, b| b.1.cmp(&a.1));
        users
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Complex Data Structures Example ===\n");

    // Create shared game state
    let game_state = A::new(GameState::new());

    // Add some initial users
    game_state.update(|state| {
        state.add_user(User::new(1, "Alice", 100));
        state.add_user(User::new(2, "Bob", 80));
        state.add_user(User::new(3, "Carol", 120));
    });

    println!("Initial game state:");
    println!("{:#?}\n", game_state.get());

    // Simulate multiple players updating scores concurrently
    let mut handles = vec![];

    // Alice gains points
    handles.push(tspawn!(mut game_state, {
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        game_state.update_user_score(1, 25);
        println!("Alice gained 25 points!");
    }));

    // Bob gains points
    handles.push(tspawn!(mut game_state, {
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        game_state.update_user_score(2, 40);
        println!("Bob gained 40 points!");
    }));

    // Carol loses points
    handles.push(tspawn!(mut game_state, {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        game_state.update_user_score(3, -15);
        println!("Carol lost 15 points!");
    }));

    // Someone checking the leaderboard
    handles.push(tspawn!(ref game_state, {
        tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
        let leaderboard = game_state.get_leaderboard();
        println!("Current leaderboard:");
        for (i, (user, score)) in leaderboard.iter().enumerate() {
            println!("  {}. {} - {} points", i + 1, user.name, score);
        }
    }));

    // Game round manager
    handles.push(tspawn!(mut game_state, {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        game_state.current_round += 1;
        println!("Advanced to round {}", game_state.current_round);
    }));

    // Wait for all tasks to complete
    for handle in handles {
        handle.await?;
    }

    println!("\n--- Final Game State ---");
    let final_state = game_state.get();
    println!("Round: {}", final_state.current_round);
    println!("Game Active: {}", final_state.game_active);

    let final_leaderboard = final_state.get_leaderboard();
    println!("\nFinal Leaderboard:");
    for (i, (user, score)) in final_leaderboard.iter().enumerate() {
        println!(
            "  {}. {} (ID: {}) - {} points",
            i + 1,
            user.name,
            user.id,
            score
        );
    }

    // Demonstrate working with nested data
    println!("\n--- Working with Nested Data ---");

    // Create a complex nested structure
    let app_data = A::new(HashMap::<String, A<Vec<String>>>::new());

    // Initialize some data
    app_data.update(|data| {
        data.insert("logs".to_string(), A::new(vec![]));
        data.insert("messages".to_string(), A::new(vec![]));
    });

    // Add logs from different tasks
    let logs_ref = {
        let data = app_data.read();
        data.get("logs").unwrap().clone()
    };

    tspawn!(logs_ref, {
        logs_ref.update(|logs| logs.push("System started".to_string()));
        println!("Added system log");
    })
    .await?;

    let logs_ref2 = {
        let data = app_data.read();
        data.get("logs").unwrap().clone()
    };

    tspawn!(logs_ref2, {
        logs_ref2.update(|logs| logs.push("User connected".to_string()));
        println!("Added user log");
    })
    .await?;

    // Read final logs
    let final_logs = {
        let data = app_data.read();
        data.get("logs").unwrap().get()
    };

    println!("Final logs: {:?}", final_logs);

    Ok(())
}
