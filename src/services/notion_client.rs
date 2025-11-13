/// Notion API client service
///
/// This module provides a high-level client for interacting with the Notion API.
/// It handles all API communication including:
/// - Authentication and connection testing
/// - Creating book pages with metadata and cover images
/// - Fetching existing books from the database
/// - Uploading and managing cover images
use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use reqwest::Client;
use serde_json::{json, Value};

use crate::models::{BookInfo, Config, NotionPage};

/// Notion API client
///
/// This struct encapsulates all Notion API operations and maintains
/// the necessary configuration and HTTP client for making requests.
pub struct NotionClient {
    /// HTTP client for making API requests
    client: Client,

    /// Notion API authentication token
    api_key: String,

    /// Target Notion database ID
    database_id: String,

    /// Name of the Notion property for book titles
    title_property: String,

    /// Name of the Notion property for author names
    author_property: String,

    /// Name of the Notion property for cover images
    cover_property: String,
}

impl NotionClient {
    /// Create a new Notion API client from configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration containing Notion credentials
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use notionshelf::models::Config;
    /// use notionshelf::services::notion_client::NotionClient;
    ///
    /// let config = Config::load().unwrap();
    /// let client = NotionClient::new(&config);
    /// ```
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            api_key: config.notion_api_key.clone(),
            database_id: config.notion_database_id.clone(),
            title_property: config.notion_title_property.clone(),
            author_property: config.notion_author_property.clone(),
            cover_property: config.notion_cover_property.clone(),
        }
    }

    /// Test connection to Notion API and database access
    ///
    /// This method verifies:
    /// - API key is valid
    /// - Database exists and is accessible
    /// - Integration has proper permissions
    ///
    /// It's recommended to call this during application initialization
    /// to provide early feedback on configuration issues.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if connection successful, detailed error otherwise
    ///
    /// # Errors
    ///
    /// Returns descriptive errors for common issues:
    /// - 404: Database not found or not shared with integration
    /// - 401: Invalid or missing API key
    /// - 403: Integration lacks required permissions
    pub async fn test_connection(&self) -> Result<()> {
        let url = format!("https://api.notion.com/v1/databases/{}", self.database_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Notion-Version", "2022-06-28")
            .send()
            .await
            .context("Failed to test Notion API connection")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            let error_message = match status.as_u16() {
                404 => format!(
                    "❌ Database not found (404):\n{}\n\n\
                    🔧 Possible solutions:\n\
                    1. Check that the Database ID is correct\n\
                    2. Share the database with your integration (Share → Add integration)\n\
                    3. Remove dashes from Database ID if present\n\
                    4. Run ./find_notion_databases.sh to find available databases",
                    error_text
                ),
                401 => format!(
                    "❌ Unauthorized access (401):\n{}\n\n\
                    🔧 Solution:\n\
                    1. Check that NOTION_API_KEY is correct\n\
                    2. Make sure the key starts with 'secret_'\n\
                    3. Create a new key at https://www.notion.so/my-integrations",
                    error_text
                ),
                403 => format!(
                    "❌ Access forbidden (403):\n{}\n\n\
                    🔧 Solution:\n\
                    1. Share the database with your integration\n\
                    2. Check integration permissions in Notion",
                    error_text
                ),
                _ => format!(
                    "❌ Notion API connection error ({}):\n{}\n\n\
                    💡 Run ./test_notion_connection.sh for diagnostics",
                    status, error_text
                ),
            };

            return Err(color_eyre::eyre::eyre!(error_message));
        }

        Ok(())
    }

    /// Create a new book page in the Notion database
    ///
    /// This method performs a complete book addition workflow:
    /// 1. Uploads cover image (if available)
    /// 2. Creates a new page in the database with book metadata
    /// 3. Sets the cover image on the page
    /// 4. Optionally adds a user comment as page content
    ///
    /// # Arguments
    ///
    /// * `book` - Book information extracted from EPUB
    /// * `user_comment` - Optional user review/comment to add to the page
    ///
    /// # Returns
    ///
    /// Returns `NotionPage` containing the ID and URL of the created page
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - API request fails
    /// - Cover image upload fails (non-fatal, page still created)
    /// - Response cannot be parsed
    pub async fn create_book_page(
        &self,
        book: &BookInfo,
        user_comment: &str,
    ) -> Result<NotionPage> {
        let url = "https://api.notion.com/v1/pages";

        // Step 1: Upload cover image if available
        let cover_file_id = if let Some(cover_data) = &book.cover_image {
            match self.upload_cover_image(cover_data).await {
                Ok(file_id) => Some(file_id),
                Err(_e) => {
                    // Cover upload failed, but we'll still create the page without it
                    None
                }
            }
        } else {
            None
        };

        // Step 2: Prepare page properties using configured property names
        let mut properties = serde_json::Map::new();

        // Add title property
        properties.insert(
            self.title_property.clone(),
            json!({
                "title": [
                    {
                        "text": {
                            "content": book.title
                        }
                    }
                ]
            }),
        );

        // Add author property
        properties.insert(
            self.author_property.clone(),
            json!({
                "rich_text": [
                    {
                        "text": {
                            "content": book.author
                        }
                    }
                ]
            }),
        );

        // Add cover property if we successfully uploaded the image
        if let Some(ref file_id) = cover_file_id {
            properties.insert(
                self.cover_property.clone(),
                json!({
                    "files": [
                        {
                            "type": "file_upload",
                            "file_upload": {
                                "id": file_id
                            },
                            "name": "cover.jpg"
                        }
                    ]
                }),
            );
        }

        // Step 3: Prepare page content (user comment if provided)
        let children = if !user_comment.is_empty() {
            vec![json!({
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [
                        {
                            "type": "text",
                            "text": {
                                "content": user_comment
                            }
                        }
                    ]
                }
            })]
        } else {
            vec![]
        };

        // Step 4: Create the page
        let payload = json!({
            "parent": {
                "database_id": self.database_id
            },
            "properties": properties,
            "children": children
        });

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("Notion-Version", "2022-06-28")
            .json(&payload)
            .send()
            .await
            .context("Failed to send request to Notion API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(color_eyre::eyre::eyre!(
                "Notion API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let response_json: Value = response
            .json()
            .await
            .context("Failed to parse Notion API response")?;

        let page_id = response_json["id"]
            .as_str()
            .context("Missing page ID in Notion response")?
            .to_string();

        let page_url = response_json["url"]
            .as_str()
            .context("Missing page URL in Notion response")?
            .to_string();

        // Step 5: Set cover image on the page if we have one
        if let Some(ref file_id) = cover_file_id {
            // Try to set page cover, but don't fail if it doesn't work
            let _ = self.set_page_cover(&page_id, file_id).await;
        }

        Ok(NotionPage {
            id: page_id,
            url: page_url,
        })
    }

    /// Upload a cover image to Notion
    ///
    /// This is a two-step process in Notion's API:
    /// 1. Create a file upload object
    /// 2. Upload the actual file data
    ///
    /// # Arguments
    ///
    /// * `image_data` - Raw bytes of the image file
    ///
    /// # Returns
    ///
    /// Returns the file upload ID that can be used to reference this image
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Image format cannot be detected
    /// - File upload object creation fails
    /// - Actual file upload fails
    pub async fn upload_cover_image(&self, image_data: &[u8]) -> Result<String> {
        // Detect image format from the file's magic bytes
        let (content_type, file_extension) = detect_image_format(image_data)?;
        let filename = format!("cover.{}", file_extension);

        // Step 1: Create file upload object
        let create_url = "https://api.notion.com/v1/file_uploads";

        let create_payload = json!({
            "filename": filename,
            "content_type": content_type
        });

        let create_response = self
            .client
            .post(create_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("Notion-Version", "2022-06-28")
            .json(&create_payload)
            .send()
            .await
            .context("Failed to create file upload object")?;

        if !create_response.status().is_success() {
            let status = create_response.status();
            let error_text = create_response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(color_eyre::eyre::eyre!(
                "Failed to create file upload object with status {}: {}",
                status,
                error_text
            ));
        }

        let create_json: Value = create_response
            .json()
            .await
            .context("Failed to parse file upload creation response")?;

        let file_upload_id = create_json["id"]
            .as_str()
            .context("Missing file upload ID in response")?
            .to_string();

        let upload_url = create_json["upload_url"]
            .as_str()
            .context("Missing upload URL in response")?;

        // Step 2: Upload the actual file data
        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(image_data.to_vec())
                .file_name(filename.clone())
                .mime_str(&content_type)?,
        );

        let upload_response = self
            .client
            .post(upload_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Notion-Version", "2022-06-28")
            .multipart(form)
            .send()
            .await
            .context("Failed to upload file data")?;

        if !upload_response.status().is_success() {
            let status = upload_response.status();
            let error_text = upload_response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(color_eyre::eyre::eyre!(
                "Failed to upload file data with status {}: {}",
                status,
                error_text
            ));
        }

        Ok(file_upload_id)
    }

    /// Set a cover image on an existing Notion page
    ///
    /// This updates the page's cover property using a previously uploaded file.
    ///
    /// # Arguments
    ///
    /// * `page_id` - ID of the Notion page to update
    /// * `file_upload_id` - ID of the uploaded file (from `upload_cover_image`)
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails
    pub async fn set_page_cover(&self, page_id: &str, file_upload_id: &str) -> Result<()> {
        let url = format!("https://api.notion.com/v1/pages/{}", page_id);

        let payload = json!({
            "cover": {
                "type": "file_upload",
                "file_upload": {
                    "id": file_upload_id
                }
            }
        });

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("Notion-Version", "2022-06-28")
            .json(&payload)
            .send()
            .await
            .context("Failed to set page cover")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(color_eyre::eyre::eyre!(
                "Failed to set page cover with status {}: {}",
                status,
                error_text
            ));
        }

        Ok(())
    }
}

/// Detect image format from file magic bytes
///
/// This function inspects the first few bytes of an image file
/// to determine its format without relying on file extensions.
///
/// Supported formats:
/// - JPEG (FF D8 FF)
/// - PNG (89 50 4E 47...)
/// - GIF (GIF87a, GIF89a)
/// - WebP (RIFF...WEBP)
/// - BMP (BM)
///
/// # Arguments
///
/// * `data` - Raw bytes of the image file
///
/// # Returns
///
/// Returns a tuple of (MIME type, file extension)
///
/// # Errors
///
/// Returns an error if:
/// - Data is too short to analyze
/// - Format cannot be determined (defaults to JPEG)
fn detect_image_format(data: &[u8]) -> Result<(String, String)> {
    if data.len() < 12 {
        return Err(color_eyre::eyre::eyre!(
            "Image data too short to determine format"
        ));
    }

    // Check for JPEG signature
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok(("image/jpeg".to_string(), "jpg".to_string()));
    }

    // Check for PNG signature
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Ok(("image/png".to_string(), "png".to_string()));
    }

    // Check for GIF signature
    if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        return Ok(("image/gif".to_string(), "gif".to_string()));
    }

    // Check for WebP signature
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return Ok(("image/webp".to_string(), "webp".to_string()));
    }

    // Check for BMP signature
    if data.starts_with(b"BM") {
        return Ok(("image/bmp".to_string(), "bmp".to_string()));
    }

    // Default to JPEG if we can't detect the format
    Ok(("image/jpeg".to_string(), "jpg".to_string()))
}
