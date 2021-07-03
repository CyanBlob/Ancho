use iced::{Application, Settings};
use paprika_api::api;
use std::env;

mod app;
//mod sample;


async fn login() -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(email) = env::var("PAPRIKA_EMAIL") {
        if let Ok(password) = env::var("PAPRIKA_PASSWORD") {
            let res = api::login(&email, &password).await;
            match res {
                Ok(t) => {
                    println!("Yay! Token: {}", t.token);
                    Ok(t.token)
                }
                Err(e) => Err(e.into()),
            }
        } else {
            Err("No password found; is the PAPRIKA_PASSWORD environment variable set?".into())
        }
    } else {
        Err("No email found; is the PAPRIKA_EMAIL environment variable set?".into())
    }
}

// print all recipes (can be a lot of requests)
#[allow(dead_code)]
async fn list_recipes(token: &str) {
    let recipe_list = api::get_recipes(&token).await.unwrap();
    for (_, recipe_entry) in recipe_list.iter().enumerate() {
        let recipe_future = api::get_recipe_by_id(&token, &recipe_entry.uid).await;
        match recipe_future {
            Ok(recipe) => println!("Recipe: {:?}", recipe),
            Err(e) => println!("Error fetching recipe {}: {}", recipe_entry.uid, e),
        }
    }
}

#[allow(dead_code)]
async fn update_recipe(token: &str, id: &str) {
    let mut recipe = api::get_recipe_by_id(&token, &id).await.unwrap();

    recipe.name = String::from("Birria tacos");
    let success = api::upload_recipe(&token, &mut recipe).await.unwrap();

    if success {
        let recipe_after_edit = api::get_recipe_by_id(&token, &recipe.uid).await.unwrap();
        println!("Edited recipe: \n{:?}", recipe_after_edit);
    } else {
        println!("Failed to update recipe");
    }
}

#[allow(dead_code)]
async fn create_recipe(token: &str) {
    let mut recipe = api::Recipe {
        uid: "".into(),
        name: "New recipe".into(),
        ingredients: "None!".into(),
        directions: "None!".into(),
        description: "None!".into(),
        notes: "".into(),
        nutritional_info: "".into(),
        servings: "".into(),
        difficulty: "".into(),
        prep_time: "".into(),
        cook_time: "".into(),
        total_time: "".into(),
        source: "acozykitchen.com".into(),
        source_url: Some("https://www.acozykitchen.com/birria-tacos".into()),
        image_url: Some("https://www.acozykitchen.com/wp-content/uploads/2021/01/BirriaTacos-11-1227x1536-2-500x500.jpg".into()),
        photo: Some("CB5F52D6-74FF-499D-8793-5FFC8190C6DC.jpg".into()),
        photo_hash: Some("36E72B4585E7ECD10AC6EF5B331789E7004BDB1F9607BC22BE27759CDD143FB6".into()),
        photo_large: None,
        scale: None,
        hash: "".into(),
        categories: vec!(),
        rating: 1,
        in_trash: false,
        is_pinned: false,
        on_favorites: false,
        on_grocery_list: false,
        created: "2021-04-09 15:09:26".into(),
        photo_url: Some("photo".into()),
    };

    recipe.uid = "".into();

    let success = api::upload_recipe(&token, &mut recipe).await.unwrap();

    if success {
        // `upload_recipe` generates a UID for us
        let recipe_after_upload = api::get_recipe_by_id(&token, &recipe.uid).await.unwrap();
        println!("New recipe: \n{:?}", recipe_after_upload);
    } else {
        println!("Failed to create recipe");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(_token) = login().await {
        println!("Login successful!");
        let res = app::HomePage::run(Settings::default());
        match res {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        }
        /*let categories = api::get_categories(&_token).await;
        for (_, category) in categories.iter().enumerate() {
            println!("Category: {:?}", category);
        }
        list_recipes(&_token).await;*/
        //update_recipe(&_token, "FD9A4450-8768-41E5-9121-3658A7411AB0".into()).await;
        //create_recipe(&_token).await;
    } else {
        return Err("Login failed!".into());
    }
}
