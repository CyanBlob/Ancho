use crate::app::paprika;

use serde::{Deserialize, Serialize};
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
        Err(e) => return None,
        Ok(recipe_string) => {
            let recipe: Recipe = serde_json::from_str(&recipe_string).unwrap();

            if recipe.hash == hash {
                println!("Got recipe from cache!");
                return Some(recipe);
            } else {
                println!("Failed to load recipe from cache :(");
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

    /*let mut file = match fs::File::open(&path) {
        Err(e) => panic!("Couldn't open file: {} {}", &file_path, e),
        Ok(file) => fs::write(path, serialized),
    };*/
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
                        let mut _paprika = paprika.lock().unwrap();

                        if _paprika.recipe_entries.len() != 0
                            && _paprika.last_fetched != _paprika.recipe_entries.len()
                        {
                            uid = _paprika.recipe_entries[_paprika.last_fetched]
                                .uid
                                .to_owned();
                            hash = _paprika.recipe_entries[_paprika.last_fetched]
                                .hash
                                .to_owned();
                            _paprika.last_fetched += 1;

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
                            // delay polling after all known recipes have been fetched
                            thread::sleep(time::Duration::from_millis(60000));
                        }
                    }
                }
                Some((recipe, paprika))
            },
        ))
    }
}
