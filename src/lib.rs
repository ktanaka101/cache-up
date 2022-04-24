use chrono::{DateTime, Duration, Utc};
use std::fmt::Debug;
use std::{collections::HashMap, hash::Hash};

pub trait Key: Clone + Eq + Hash + Debug {}

pub trait Value: Clone + Debug {}

pub struct CacheUp<K: Key, V: Value> {
    store: HashMap<K, (V, CacheContext<K, V>)>,
}

impl<K: Key, V: Value> Debug for CacheUp<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CacheUp {{ ")?;
        for (k, v) in self.store.iter() {
            write!(f, "{:?} => ({:?}), ", k, v.0)?;
        }
        write!(f, "}}")
    }
}

impl<K: Key, V: Value> Default for CacheUp<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CacheContext<K: Key, V: Value> {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub option: CacheOption<K, V>,
}

#[derive(Default)]
pub struct CacheOption<K, V>
where
    K: Key,
    V: Value,
{
    #[allow(clippy::type_complexity)]
    policies: Vec<Box<dyn Fn(&K, &V, &CacheContext<K, V>) -> bool>>,
}

impl<K: Key, V: Value> CacheOption<K, V> {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    pub fn add_policy<F: 'static + Fn(&K, &V, &CacheContext<K, V>) -> bool>(
        mut self,
        policy: F,
    ) -> Self {
        self.policies.push(Box::new(policy));
        self
    }

    pub fn max_age(self, max_age: i64) -> Self {
        self.add_policy(move |_, _, ctx| Self::_max_age(max_age, Utc::now(), ctx))
    }

    fn _max_age(max_age: i64, now: DateTime<Utc>, context: &CacheContext<K, V>) -> bool {
        let updated_at = context.updated_at;
        let diff_updated = now.signed_duration_since(updated_at);
        let max_age = Duration::seconds(max_age);

        diff_updated < max_age
    }
}

impl<K: Key, V: Value> CacheUp<K, V> {
    pub fn new() -> CacheUp<K, V> {
        CacheUp {
            store: HashMap::new(),
        }
    }

    pub fn execute<F: Fn() -> V>(&mut self, key: K, f: F) -> &(V, CacheContext<K, V>) {
        self.execute_with_option(key, f, CacheOption::<K, V>::new())
    }

    pub fn execute_with_option<F: Fn() -> V>(
        &mut self,
        key: K,
        f: F,
        option: CacheOption<K, V>,
    ) -> &(V, CacheContext<K, V>) {
        self.store
            .entry(key.clone())
            .and_modify(|item| {
                for policy in &item.1.option.policies {
                    if policy(&key, &item.0, &item.1) {
                        item.0 = f();
                        item.1.updated_at = Utc::now();
                        break;
                    }
                }
            })
            .or_insert_with(|| {
                let result = f();
                let created_at = Utc::now();
                let cache_context = CacheContext {
                    created_at,
                    updated_at: created_at,
                    option,
                };

                (result, cache_context)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_for_i64() {
        impl Key for i64 {}
        impl Value for i64 {}

        let mut cache_up = CacheUp::<i64, i64>::new();
        let (result, _) = cache_up.execute(1, || 2 + 2);
        assert_eq!(result, &4);

        let (result, _) = cache_up.execute(1, || 5 + 5);
        assert_eq!(result, &4);

        let (result, _) = cache_up.execute(2, || 5 + 5);
        assert_eq!(result, &10);

        let (result, _) = cache_up.execute(2, || 6 + 6);
        assert_eq!(result, &10);
    }

    #[test]
    fn it_works_for_enum() {
        #[derive(Clone, Debug, PartialEq, Eq)]
        enum Test {
            A,
            B,
            C(String),
        }
        impl Value for Test {}
        impl Key for String {}

        let mut cache_up = CacheUp::<String, Test>::new();
        let (result, _) = cache_up.execute("aaa".to_string(), || Test::A);
        assert_eq!(result, &Test::A);

        let (result, _) = cache_up.execute("aaa".to_string(), || Test::B);
        assert_eq!(result, &Test::A);

        let (result, _) = cache_up.execute("bbb".to_string(), || Test::B);
        assert_eq!(result, &Test::B);

        let (result, _) = cache_up.execute("ccc".to_string(), || Test::C("inner_ccc".to_string()));
        assert_eq!(result, &Test::C("inner_ccc".to_string()));
    }

    #[test]
    fn it_works_with_option() {
        let mut cache_up = CacheUp::<i64, i64>::new();
        let cache_opt = CacheOption::new().add_policy(|_, _, _| true);
        let (result, _) = cache_up.execute_with_option(1, || 2 + 2, cache_opt);
        assert_eq!(result, &4);

        let (result, _) = cache_up.execute(1, || 5 + 5);
        assert_eq!(result, &10);

        let mut cache_up = CacheUp::<i64, i64>::new();
        let cache_opt = CacheOption::new().add_policy(|_, _, _| false);
        let (result, _) = cache_up.execute_with_option(1, || 2 + 2, cache_opt);
        assert_eq!(result, &4);

        let (result, _) = cache_up.execute(1, || 5 + 5);
        assert_eq!(result, &4);
    }
}
