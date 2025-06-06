use crate::consts;
use async_trait::async_trait;

#[derive(Clone)]
pub struct StorageHandler {
    pub client: aws_sdk_s3::Client,
}

#[async_trait]
impl crate::services::StorageService for StorageHandler {
    async fn save_pic(
        &self,
        user_email: &str,
        file_name: &str,
        body: Vec<u8>,
    ) -> anyhow::Result<()> {
        let body = aws_sdk_s3::primitives::ByteStream::from(body);
        let pic_path = format!("pics/{user_email}/{file_name}").to_lowercase();

        self.client
            .put_object()
            .bucket(consts::S3_MAIN_BUCKET_NAME)
            .key(pic_path)
            .body(body)
            .send()
            .await?;

        Ok(())
    }

    async fn get_pic_as_bytes(&self, file_name: &str) -> anyhow::Result<Vec<u8>> {
        let object = self
            .client
            .get_object()
            .bucket(consts::S3_MAIN_BUCKET_NAME)
            .key(file_name)
            .send()
            .await?;

        Ok(object
            .body
            .collect()
            .await
            .map(|package| package.into_bytes())?
            .into_iter()
            .collect::<Vec<u8>>())
    }
}
