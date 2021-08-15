use crate::app::paprika;

use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::{fs, path, thread, time};

use iced_futures::futures;

use paprika_api::api::Recipe;

const CACHE_DIR: &str = "recipe_cache";

pub struct RecipeFetcher<T> {
    pub id: T,
    pub paprika: Arc<Mutex<paprika::Paprika>>,
}

fn get_recipe_from_cache(uid: &str, hash: &str) -> Option<Recipe> {
    if !path::Path::new(CACHE_DIR).is_dir() {
        return None;
    }

    let file_path = format!(r#"{}/{}"#, CACHE_DIR, uid);

    let path = path::Path::new(&file_path);

    match fs::read_to_string(&path) {
        Err(_) => {
            println!("Failed to load recipe from cache :(");
            return None;
        }
        Ok(recipe_string) => {
            let recipe: Recipe = serde_json::from_str(&recipe_string).unwrap();

            if recipe.hash == hash {
                println!("Got recipe from cache!");
                return Some(recipe);
            } else {
                println!("Recipe changed; cache invalidated");
                fs::remove_file(&path).unwrap();
                return None;
            }
        }
    };
}

#[allow(unused)]
fn save_recipe_to_cache(recipe: Option<&Recipe>) -> Result<(), std::io::Error> {
    if !path::Path::new(CACHE_DIR).is_dir() {
        fs::create_dir(CACHE_DIR)?;
    }

    let serialized = serde_json::to_string(recipe.unwrap()).unwrap();

    let file_path = format!(r#"{}/{}"#, CACHE_DIR, &recipe.unwrap().uid);

    let path = path::Path::new(&file_path);

    fs::write(path, serialized);

    Ok(())
}

impl<H, I, T> iced_native::subscription::Recipe<H, I> for RecipeFetcher<T>
where
    T: 'static + Hash + Copy + Send,
    H: Hasher,
{
    type Output = Option<Recipe>;

    fn hash(&self, state: &mut H) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.id.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<I>,
    ) -> futures::stream::BoxStream<Self::Output> {
        let _id = self.id;

        Box::pin(futures::stream::unfold(
            self.paprika.clone(),
            move |paprika| async move {
                let uid;
                let hash;
                let mut recipe = None;
                {
                    // update recipe list
                    {
                        let mut _paprika = paprika.lock().unwrap();

                        if _paprika.recipe_entries.len() == 0 {
                            tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()
                                .unwrap()
                                .block_on(_paprika.fetch_recipe_list());
                        }
                    }

                    {
                        let recipe_count;
                        let last_fetched;
                        {
                            let mut _paprika = paprika.lock().unwrap();
                            recipe_count = _paprika.recipe_entries.len();
                            last_fetched = _paprika.last_fetched;
                        }

                        if recipe_count != 0 && last_fetched != recipe_count {
                            let mut _paprika = paprika.lock().unwrap();

                            uid = _paprika.recipe_entries[_paprika.last_fetched]
                                .uid
                                .to_owned();
                            hash = _paprika.recipe_entries[_paprika.last_fetched]
                                .hash
                                .to_owned();
                            _paprika.last_fetched += 1;

                            println!("Fetching recipe: {}", _paprika.last_fetched);
                            match get_recipe_from_cache(&uid, &hash) {
                                Some(cached_recipe) => recipe = Some(cached_recipe),
                                None => {
                                    recipe = Some(
                                        tokio::runtime::Builder::new_current_thread()
                                            .enable_all()
                                            .build()
                                            .unwrap()
                                            .block_on(_paprika.get_recipe_by_id(&uid)),
                                    );

                                    match save_recipe_to_cache(recipe.as_ref()) {
                                        Err(e) => println!("Failed to save recipe to cache: {}", e),
                                        _ => (),
                                    }
                                }
                            }
                        } else {
                            // check for updated recipes every minute after fetching them all
                            thread::sleep(time::Duration::from_millis(5000));
                            println!("Re-fetching recipes!");
                            {
                                let mut _paprika = paprika.lock().unwrap();
                                _paprika.last_fetched = 0;
                                _paprika.recipe_entries.clear();
                            }
                        }
                    }
                }
                Some((recipe, paprika))
            },
        ))
    }
}
