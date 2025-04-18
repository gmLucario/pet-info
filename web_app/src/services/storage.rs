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

        self.client
            .put_object()
            .bucket(consts::S3_MAIN_BUCKET_NAME)
            .key(format!("pics/{}/{}", user_email, file_name))
            .body(body)
            .send()
            .await?;

        Ok(())
    }

    async fn get_pic_as_bytes(&self, file_name: &str) -> anyhow::Result<Vec<u8>> {
        let mut object = self
            .client
            .get_object()
            .bucket(consts::S3_MAIN_BUCKET_NAME)
            .key(file_name)
            .send()
            .await?;

        let mut body: Vec<u8> = vec![];

        while let Ok(Some(bytes)) = object.body.try_next().await {
            body.extend_from_slice(&bytes.into_iter().collect::<Vec<u8>>());
        }

        Ok(body)
    }
}
