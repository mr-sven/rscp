use crate::item::Item;
use anyhow::{anyhow, Result};
use std::any::{Any, TypeId};

/// Item and data getter for Frame and Item
pub trait GetItem {
    /// returns typed data from data property
    ///
    /// # Examples
    /// ```
    /// use rscp::{tags, Item, GetItem};
    /// let item = Item::new(tags::RSCP::AUTHENTICATION_USER.into(), "username".to_string());
    /// assert_eq!(item.get_data::<String>().unwrap(), "username");
    /// ```
    fn get_data<T: 'static + Sized>(&self) -> Result<&T>;

    /// returns item by tag from data / item list
    ///
    /// # Arguments
    ///
    /// * `tag` - Tag Identifier
    ///
    /// # Examples
    /// ```
    /// use rscp::{tags, Item, GetItem};
    /// let item_container = Item::new(tags::RSCP::AUTHENTICATION.into(), vec![
    ///     Item::new(tags::RSCP::AUTHENTICATION_USER.into(), self.username.to_string()),
    ///     Item::new(tags::RSCP::AUTHENTICATION_PASSWORD.into(), self.password.to_string()),
    /// ]);
    /// let item = item_container.get_item(tags::RSCP::AUTHENTICATION_USER.into()).unwrap();
    /// ```
    fn get_item(&self, tag: u32) -> Result<&Item>;

    /// returns typed item data by tag from data / item list
    ///
    /// # Arguments
    ///
    /// * `tag` - Tag Identifier
    ///
    /// # Examples
    /// ```
    /// use rscp::{tags, Item, GetItem};
    /// let item_container = Item::new(tags::RSCP::AUTHENTICATION.into(), vec![
    ///     Item::new(tags::RSCP::AUTHENTICATION_USER.into(), self.username.to_string()),
    ///     Item::new(tags::RSCP::AUTHENTICATION_PASSWORD.into(), self.password.to_string()),
    /// ]);
    /// assert_eq!(item_container.get_item_data::<String>(tags::RSCP::AUTHENTICATION_USER.into()).unwrap(), "username");
    /// ```
    fn get_item_data<T: 'static + Sized>(&self, tag: u32) -> Result<&T>;
}

/// implementation for data object
impl GetItem for Option<Box<dyn Any>> {
    fn get_data<T: 'static + Sized>(&self) -> Result<&T> {
        Ok(self.as_ref().unwrap().as_ref().downcast_ref::<T>().unwrap())
    }

    fn get_item(&self, tag: u32) -> Result<&Item> {
        let items = self.as_ref().unwrap().downcast_ref::<Vec<Item>>().unwrap();
        for item in items {
            if item.tag == tag {
                return Ok(item);
            }
        }
        Err(anyhow!("Tag not found {:?}", tag))
    }

    fn get_item_data<T: 'static + Sized>(&self, tag: u32) -> Result<&T> {
        let item = self.get_item(tag)?;
        Ok(item
            .data
            .as_ref()
            .unwrap()
            .as_ref()
            .downcast_ref::<T>()
            .unwrap())
    }
}
