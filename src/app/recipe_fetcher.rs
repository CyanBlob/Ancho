use crate::app::paprika;

use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::{thread, time};

use iced_futures::futures;

pub struct RecipeFetcher<T> {
    pub id: T,
    pub paprika: Arc<Mutex<paprika::Paprika>>,
}

impl<H, I, T> iced_native::subscription::Recipe<H, I> for RecipeFetcher<T>
where
    T: 'static + Hash + Copy + Send,
    H: Hasher,
{
    type Output = Option<paprika_api::api::Recipe>;

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
                let mut recipe = None;
                {
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
                            _paprika.last_fetched += 1;

                            recipe = Some(tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()
                                .unwrap()
                                .block_on(_paprika.get_recipe_by_id(&uid)));
                        }
                        else {
                            thread::sleep(time::Duration::from_millis(5000));
                        }
                    }
                }
                Some((recipe, paprika))
            },
        ))
    }
}