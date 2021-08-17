use crate::app::account::Account;
use paprika_api::api;
use std::env;

pub struct Paprika {
    pub token: String,
    pub recipe_entries: Vec<api::RecipeEntry>,
    pub last_fetched: usize,
    pub account: Account,
    account_hash: String,
}

impl Paprika {
    pub fn new() -> Self {
        Self {
            token: "".into(),
            recipe_entries: Vec::new(),
            last_fetched: 0,
            account: Account::new("".into(), "".into()),
            account_hash: "".into(),
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

    pub async fn account_login(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let res = paprika_api::api::login(&self.account.username, &self.account.password).await;
        match res {
            Ok(t) => {
                println!("Yay! Token: {}", t.token);
                self.token = t.token.into();
                Ok(self.token.to_owned())
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn fetch_recipe_list(&mut self) {
        if self.account.get_hash() != self.account_hash
            && !self.account.username.is_empty()
            && !self.account.password.is_empty()
        {
            self.account_hash = self.account.get_hash();

            let login_ret = self.account_login().await;
            match login_ret {
                Ok(_) => println!("Logged in with account!"),
                Err(_) => println!("Couldn't log in with account"),
            }
        } else if self.token.is_empty() {
            let login_ret = self.login().await;
            match login_ret {
                Ok(_) => println!("Logged in!"),
                Err(_) => println!("Couldn't log in!"),
            }
        }

        if !self.token.is_empty() {
            self.recipe_entries = paprika_api::api::get_recipes(&self.token).await.unwrap();
        }
    }

    pub async fn get_recipe_by_id(&mut self, id: &str) -> paprika_api::api::Recipe {
        if self.token.is_empty() {
            self.login().await.expect("Couldn't log in");
        }
        if self.recipe_entries.len() == 0 {
            self.fetch_recipe_list().await;
        }

        if !self.token.is_empty() {
            let recipe = paprika_api::api::get_recipe_by_id(&self.token, id)
                .await
                .unwrap();
            return recipe;
        }
        return paprika_api::api::Recipe::default();
    }

    #[allow(unused)]
    pub async fn fetch_recipe_by_id(&mut self, id: &str) {
        if self.token.is_empty() {
            self.login().await.expect("Couldn't log in");
        }
        if self.recipe_entries.len() == 0 {
            self.fetch_recipe_list().await;
        }
    }

    pub async fn update_recipe(&mut self, recipe: &mut paprika_api::api::Recipe) {
        if self.token.is_empty() {
            self.login().await.expect("Couldn't log in");
        }

        recipe.hash.clear();

        let success = paprika_api::api::upload_recipe(&self.token, recipe)
            .await
            .unwrap();

        if !success {
            println!("Failed to update recipe");
        }
    }
}
