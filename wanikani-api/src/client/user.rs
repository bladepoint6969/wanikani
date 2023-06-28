use crate::{
    user::{UpdateUser, User},
    Error,
};

use super::WKClient;

const USER_PATH: &str = "user";

impl WKClient {
    /// Returns a summary of user information.
    pub async fn get_user_information(&self) -> Result<User, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut().expect("Valid URL").push(USER_PATH);

        let req = self.client.get(url);

        self.do_request("get_user_information", req).await
    }

    /// Returns an updated summary of user information.
    pub async fn update_user_information(&self, user: &UpdateUser) -> Result<User, Error> {
        let mut url = self.base_url.clone();
        url.path_segments_mut().expect("Valid URL").push(USER_PATH);

        let req = self.client.put(url).json(user);

        self.do_request("update_user_information", req).await
    }
}

#[cfg(test)]
mod tests {
    use crate::client::{init_tests, create_client};

    #[tokio::test]
    async fn test_get_user_information() {
        init_tests();

        let client = create_client();

        assert!(client.get_user_information().await.is_ok());
    }

    #[tokio::test]
    async fn test_update_user_information() {
        use crate::user::{UpdatePreferences, UpdateUser};

        init_tests();

        let client = create_client();

        let user = client.get_user_information().await.expect("Success");

        let preferences = UpdatePreferences {
            default_voice_actor_id: Some(2),
            ..user.data.preferences.into()
        };
        let mut update = UpdateUser { preferences };

        let updated_user = client
            .update_user_information(&update)
            .await
            .expect("Success");

        assert_ne!(updated_user, user);
        assert_eq!(updated_user.data.preferences.default_voice_actor_id, 2);
        assert!(
            updated_user.common.data_updated_at.expect("Timestamp")
                > user.common.data_updated_at.expect("Timestamp")
        );

        update.preferences = user.data.preferences.into();
        let reset_user = client
            .update_user_information(&update)
            .await
            .expect("Success");

        assert_eq!(reset_user.data, user.data);
        assert!(
            reset_user.common.data_updated_at.expect("Timestamp")
                > updated_user.common.data_updated_at.expect("Timestamp")
        );
    }
}