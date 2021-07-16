use paprika_api::api;
use std::env;

pub struct Paprika {
    token: String,
    pub recipe_entries: Vec<api::RecipeEntry>,
    pub recipes: Vec<api::Recipe>,
    pub last_fetched: usize
}

impl Paprika {
    pub fn new() -> Self {
        Self {
            token: "".into(),
            recipe_entries: Vec::new(),
            recipes: Vec::new(),
            last_fetched: 0
        }
    }

    pub async fn login(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        if let Ok(email) = env::var("PAPRIKA_EMAIL") {
            if let Ok(password) = env::var("PAPRIKA_PASSWORD") {
                let res = paprika_api::api::login(&email, &password).await;
                match res {
                    Ok(t) => {
                        println!("Yay! Token: {}", t.token);
                        self.token = t.token.into();
                        Ok(self.token.to_owned())
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
    pub async fn list_recipes(&mut self) {
        self.fetch_recipes().await;
        for recipe in &self.recipes {
            println!("{:?}", recipe);
        }
    }

    pub async fn fetch_recipe_list(&mut self) {
        if self.token.is_empty() {
            self.login().await.expect("Couldn't log in");
        }
        self.recipe_entries = paprika_api::api::get_recipes(&self.token).await.unwrap();
    }

    #[allow(unused)]
    pub async fn get_recipe_by_id(&mut self, id: &str) -> paprika_api::api::Recipe {
        if self.token.is_empty() {
            self.login().await.expect("Couldn't log in");
        }
        if self.recipe_entries.len() == 0 {
            self.fetch_recipe_list().await;
        }
        let recipe = paprika_api::api::get_recipe_by_id(&self.token, id).await.unwrap();
        recipe
    }

    #[allow(unused)]
    pub async fn fetch_recipe_by_id(&mut self, id: &str) {
        if self.token.is_empty() {
            self.login().await.expect("Couldn't log in");
        }
        if self.recipe_entries.len() == 0 {
            self.fetch_recipe_list().await;
        }
        self.recipes.push(paprika_api::api::get_recipe_by_id(&self.token, id).await.unwrap());
    }

    pub async fn fetch_recipes(&mut self) {
        if self.token.is_empty() {
            self.login().await.expect("Couldn't log in");
        }

        self.recipes.clear();

        let recipe_list = paprika_api::api::get_recipes(&self.token).await.unwrap();
        for (_, recipe_entry) in recipe_list.iter().enumerate() {
            let recipe_future =
                paprika_api::api::get_recipe_by_id(&self.token, &recipe_entry.uid).await;
            match recipe_future {
                Ok(recipe) => {
                    if !&recipe.in_trash {
                        println!("Added recipe {}: {}", self.recipes.len(), &recipe.name);
                        self.recipes.push(recipe);
                    }
                }
                Err(e) => println!("Error fetching recipe {}: {}", recipe_entry.uid, e),
            }
        }
    }

    #[allow(dead_code)]
    pub async fn update_recipe(&mut self, id: &str) {
        if self.token.is_empty() {
            self.login().await.expect("Couldn't log in");
        }

        let mut recipe = paprika_api::api::get_recipe_by_id(&self.token, &id)
            .await
            .unwrap();

        recipe.name = String::from("Birria tacos");
        let success = paprika_api::api::upload_recipe(&self.token, &mut recipe)
            .await
            .unwrap();

        if success {
            let recipe_after_edit = paprika_api::api::get_recipe_by_id(&self.token, &recipe.uid)
                .await
                .unwrap();
            println!("Edited recipe: \n{:?}", recipe_after_edit);
        } else {
            println!("Failed to update recipe");
        }
    }

    #[allow(dead_code)]
    pub async fn create_recipe(&mut self) {
        if self.token.is_empty() {
            self.login().await.expect("Couldn't log in");
        }

        let mut recipe = paprika_api::api::Recipe {
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

        let success = paprika_api::api::upload_recipe(&self.token, &mut recipe)
            .await
            .unwrap();

        if success {
            // `upload_recipe` generates a UID for us
            let recipe_after_upload = paprika_api::api::get_recipe_by_id(&self.token, &recipe.uid)
                .await
                .unwrap();
            println!("New recipe: \n{:?}", recipe_after_upload);
        } else {
            println!("Failed to create recipe");
        }
    }
}

/*#[tokio::main]
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
}*/
