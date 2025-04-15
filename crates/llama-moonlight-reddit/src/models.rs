//! Models for Reddit API types
//!
//! This module contains data structures that represent the core entities in the Reddit API.

use std::collections::HashMap;
use std::fmt;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize, Deserializer};
use serde::de::{self, Visitor, MapAccess};
use url::Url;

/// A Reddit thing, which is any object with a unique ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thing<T> {
    /// The fullname of the thing - a combination of type prefix and ID (e.g., "t3_abcdef")
    pub name: String,
    
    /// The type of thing
    pub kind: ThingKind,
    
    /// The actual data of the thing
    pub data: T,
}

/// The type of a Reddit thing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThingKind {
    /// Comment (t1_)
    #[serde(rename = "t1")]
    Comment,
    
    /// Account (t2_)
    #[serde(rename = "t2")]
    Account,
    
    /// Link/Post (t3_)
    #[serde(rename = "t3")]
    Link,
    
    /// Message (t4_)
    #[serde(rename = "t4")]
    Message,
    
    /// Subreddit (t5_)
    #[serde(rename = "t5")]
    Subreddit,
    
    /// Award (t6_)
    #[serde(rename = "t6")]
    Award,
    
    /// Listing
    #[serde(rename = "Listing")]
    Listing,
    
    /// More comments
    #[serde(rename = "more")]
    More,
    
    /// Unknown type
    #[serde(other)]
    Unknown,
}

impl ThingKind {
    /// Get the prefix for this thing kind
    pub fn prefix(&self) -> &'static str {
        match self {
            ThingKind::Comment => "t1_",
            ThingKind::Account => "t2_",
            ThingKind::Link => "t3_",
            ThingKind::Message => "t4_",
            ThingKind::Subreddit => "t5_",
            ThingKind::Award => "t6_",
            ThingKind::Listing => "",
            ThingKind::More => "",
            ThingKind::Unknown => "",
        }
    }
    
    /// Check if a fullname matches this kind
    pub fn matches_fullname(&self, fullname: &str) -> bool {
        let prefix = self.prefix();
        !prefix.is_empty() && fullname.starts_with(prefix)
    }
    
    /// Parse a thing kind from a fullname
    pub fn from_fullname(fullname: &str) -> Option<Self> {
        if fullname.starts_with("t1_") {
            Some(ThingKind::Comment)
        } else if fullname.starts_with("t2_") {
            Some(ThingKind::Account)
        } else if fullname.starts_with("t3_") {
            Some(ThingKind::Link)
        } else if fullname.starts_with("t4_") {
            Some(ThingKind::Message)
        } else if fullname.starts_with("t5_") {
            Some(ThingKind::Subreddit)
        } else if fullname.starts_with("t6_") {
            Some(ThingKind::Award)
        } else {
            None
        }
    }
}

/// A listing of Reddit things
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing<T> {
    /// The fullname of the listing
    pub kind: String,
    
    /// The listing data
    pub data: ListingData<T>,
}

/// Data for a listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListingData<T> {
    /// Modhash (for CSRF protection)
    pub modhash: Option<String>,
    
    /// Number of children
    pub dist: Option<i32>,
    
    /// Fullname of the thing after which to retrieve more items
    pub after: Option<String>,
    
    /// Fullname of the thing before which to retrieve more items
    pub before: Option<String>,
    
    /// The listed things
    pub children: Vec<T>,
}

/// A Reddit post/link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    /// Unique ID of the post (without prefix)
    pub id: String,
    
    /// Fullname of the post (with t3_ prefix)
    pub name: String,
    
    /// The post title
    pub title: String,
    
    /// Subreddit name (without /r/ prefix)
    pub subreddit: String,
    
    /// Fullname of the subreddit (with t5_ prefix)
    pub subreddit_id: String,
    
    /// Subreddit name prefix (e.g., "r/rust")
    pub subreddit_name_prefixed: String,
    
    /// Author's username
    pub Author: Nik Jois
    
    /// Author's fullname
    pub Author: Nik Jois
    
    /// Whether the author is a moderator
    pub distinguished: Option<String>,
    
    /// The text content of the post (for self posts)
    pub selftext: String,
    
    /// The HTML content of the post (for self posts)
    pub selftext_html: Option<String>,
    
    /// URL the post links to (external or Reddit URL)
    pub url: String,
    
    /// Permalink to the post on Reddit
    pub permalink: String,
    
    /// Domain of the linked content
    pub domain: String,
    
    /// Whether the post is a self post
    pub is_self: bool,
    
    /// Whether the post contains NSFW content
    pub over_18: bool,
    
    /// Whether the post contains spoilers
    pub spoiler: bool,
    
    /// Whether the post is locked (no new comments)
    pub locked: bool,
    
    /// Whether the post is stickied
    pub stickied: bool,
    
    /// Whether the post is archived
    pub archived: bool,
    
    /// Whether the post is saved by the current user
    pub saved: bool,
    
    /// Whether the post is hidden by the current user
    pub hidden: bool,
    
    /// Whether the post has been visited by the current user
    pub visited: bool,
    
    /// Number of upvotes
    pub ups: i32,
    
    /// Number of downvotes (usually 0 due to Reddit's fuzzing)
    pub downs: i32,
    
    /// Total score (upvotes - downvotes)
    pub score: i32,
    
    /// Upvote ratio
    pub upvote_ratio: f32,
    
    /// Number of comments
    pub num_comments: i32,
    
    /// Creation time (UTC)
    #[serde(with = "timestamp_seconds")]
    pub created_utc: DateTime<Utc>,
    
    /// Whether the post has been edited
    pub edited: Edited,
    
    /// The post's flair
    pub link_flair_text: Option<String>,
    
    /// The author's flair in the subreddit
    pub Author: Nik Jois
    
    /// Thumbnail URL
    pub thumbnail: Option<String>,
    
    /// Post content (images, video, etc.)
    pub media: Option<serde_json::Value>,
    
    /// Preview images
    pub preview: Option<Preview>,
    
    /// Gallery data (for multi-image posts)
    pub gallery_data: Option<GalleryData>,
    
    /// URL containing the crosspost parent fullname
    pub crosspost_parent: Option<String>,
    
    /// Original post, if this is a crosspost
    pub crosspost_parent_list: Option<Vec<Box<Post>>>,
    
    /// Whether the post has been removed
    pub removed: Option<bool>,
    
    /// Whether the post has been removed by spam filter
    pub spam: Option<bool>,
    
    /// Whether the post has been manually approved
    pub approved: Option<bool>,
    
    /// Whether the current user is a moderator of the subreddit
    pub is_mod: Option<bool>,
    
    /// Whether the current user is a subscriber of the subreddit
    pub is_subscriber: Option<bool>,
    
    /// Additional awards data
    pub all_awardings: Vec<Award>,
    
    /// Additional custom fields returned by Reddit
    #[serde(flatten)]
    pub additional_fields: HashMap<String, serde_json::Value>,
}

/// A comment on a post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Unique ID of the comment (without prefix)
    pub id: String,
    
    /// Fullname of the comment (with t1_ prefix)
    pub name: String,
    
    /// Parent ID (fullname of parent comment or post)
    pub parent_id: String,
    
    /// ID of the post this comment is on
    pub link_id: String,
    
    /// Subreddit name (without /r/ prefix)
    pub subreddit: String,
    
    /// Subreddit name prefixed (e.g., "r/rust")
    pub subreddit_name_prefixed: String,
    
    /// Author's username
    pub Author: Nik Jois
    
    /// Author's fullname
    pub Author: Nik Jois
    
    /// Whether the author is a moderator, admin, etc.
    pub distinguished: Option<String>,
    
    /// The text content of the comment
    pub body: String,
    
    /// The HTML content of the comment
    pub body_html: Option<String>,
    
    /// Whether the comment has been edited
    pub edited: Edited,
    
    /// Permalink to the comment on Reddit
    pub permalink: String,
    
    /// Number of upvotes
    pub ups: i32,
    
    /// Number of downvotes (usually 0 due to Reddit's fuzzing)
    pub downs: i32,
    
    /// Total score (upvotes - downvotes)
    pub score: i32,
    
    /// Whether the score is hidden
    pub score_hidden: bool,
    
    /// Creation time (UTC)
    #[serde(with = "timestamp_seconds")]
    pub created_utc: DateTime<Utc>,
    
    /// Whether the comment is stickied
    pub stickied: bool,
    
    /// Whether the comment is locked
    pub locked: bool,
    
    /// Whether the comment is archived
    pub archived: bool,
    
    /// Whether the comment is saved by the current user
    pub saved: bool,
    
    /// The author's flair in the subreddit
    pub Author: Nik Jois
    
    /// Child comments, if available
    pub replies: Replies,
    
    /// Depth level in the comment tree
    pub depth: Option<i32>,
    
    /// Number of child comments
    pub count: Option<i32>,
    
    /// Children IDs, if this is a "more" object
    pub children: Option<Vec<String>>,
    
    /// Whether the comment has been removed
    pub removed: Option<bool>,
    
    /// Whether the comment has been removed by spam filter
    pub spam: Option<bool>,
    
    /// Whether the comment has been manually approved
    pub approved: Option<bool>,
    
    /// Additional awards data
    pub all_awardings: Vec<Award>,
    
    /// Additional custom fields returned by Reddit
    #[serde(flatten)]
    pub additional_fields: HashMap<String, serde_json::Value>,
}

/// Reply data for comments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Replies {
    /// A listing of comments
    Listing(Box<Listing<Thing<Comment>>>),
    
    /// Empty replies
    Empty(String),
}

/// Info about a subreddit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subreddit {
    /// Unique ID of the subreddit (without prefix)
    pub id: String,
    
    /// Fullname of the subreddit (with t5_ prefix)
    pub name: String,
    
    /// The display name of the subreddit (without /r/ prefix)
    pub display_name: String,
    
    /// The display name of the subreddit prefixed with "r/"
    pub display_name_prefixed: String,
    
    /// The subreddit's title
    pub title: String,
    
    /// Public description
    pub public_description: String,
    
    /// Full description
    pub description: String,
    
    /// HTML formatted description
    pub description_html: Option<String>,
    
    /// URL (usually "/r/[name]")
    pub url: String,
    
    /// Creation time (UTC)
    #[serde(with = "timestamp_seconds")]
    pub created_utc: DateTime<Utc>,
    
    /// Subscriber count
    pub subscribers: i32,
    
    /// Active user count
    pub active_user_count: Option<i32>,
    
    /// Whether the subreddit is NSFW
    pub over18: bool,
    
    /// Whether the subreddit is quarantined
    pub quarantine: bool,
    
    /// Whether the subreddit is restricted
    pub restrict_posting: bool,
    
    /// Whether the subreddit is private
    pub subreddit_type: SubredditType,
    
    /// Subreddit rules
    pub rules: Option<Vec<SubredditRule>>,
    
    /// Custom CSS
    pub custom_css: Option<String>,
    
    /// Subreddit header image URL
    pub header_img: Option<String>,
    
    /// Subreddit icon image URL
    pub icon_img: Option<String>,
    
    /// Subreddit banner image URL
    pub banner_img: Option<String>,
    
    /// Community icon image URL
    pub community_icon: Option<String>,
    
    /// Whether the current user is banned
    pub user_is_banned: Option<bool>,
    
    /// Whether the current user is an approved submitter
    pub user_is_contributor: Option<bool>,
    
    /// Whether the current user is a moderator
    pub user_is_moderator: Option<bool>,
    
    /// Whether the current user is a subscriber
    pub user_is_subscriber: Option<bool>,
    
    /// Whether spoilers are enabled
    pub spoilers_enabled: bool,
    
    /// Whether the subreddit allows multiple images per post
    pub allow_galleries: Option<bool>,
    
    /// Whether the subreddit allows images in posts
    pub allow_images: Option<bool>,
    
    /// Whether the subreddit allows video posts
    pub allow_videos: Option<bool>,
    
    /// Whether the subreddit allows polls
    pub allow_polls: Option<bool>,
    
    /// Link flair templates
    pub link_flair_templates: Option<Vec<FlairTemplate>>,
    
    /// Primary color
    pub primary_color: Option<String>,
    
    /// Additional custom fields returned by Reddit
    #[serde(flatten)]
    pub additional_fields: HashMap<String, serde_json::Value>,
}

/// A subreddit rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubredditRule {
    /// Rule kind (link, comment, or all)
    pub kind: String,
    
    /// Short description
    pub short_name: String,
    
    /// Full description
    pub description: String,
    
    /// Violation reason
    pub violation_reason: String,
    
    /// Created time
    pub created_utc: Option<f64>,
    
    /// Priority
    pub priority: Option<i32>,
}

/// Type of a subreddit
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubredditType {
    /// Public subreddit
    Public,
    
    /// Private subreddit
    Private,
    
    /// Restricted subreddit
    Restricted,
    
    /// Gold-only subreddit
    GoldRestricted,
    
    /// Archived subreddit
    Archived,
    
    /// Employees-only subreddit
    EmployeesOnly,
    
    /// User profile as a subreddit
    User,
}

/// A Reddit user account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique ID of the user (without prefix)
    pub id: String,
    
    /// Fullname of the user (with t2_ prefix)
    pub name: String,
    
    /// Username
    pub username: String,
    
    /// Creation time (UTC)
    #[serde(with = "timestamp_seconds")]
    pub created_utc: DateTime<Utc>,
    
    /// Comment karma
    pub comment_karma: i32,
    
    /// Link/post karma
    pub link_karma: i32,
    
    /// Total karma (comment + link)
    pub total_karma: Option<i32>,
    
    /// Whether the account has Reddit Premium
    pub is_gold: bool,
    
    /// Whether the account is a moderator
    pub is_mod: bool,
    
    /// Whether the account is verified as an employee
    pub verified: bool,
    
    /// Whether the account has a verified email
    pub has_verified_Email: nikjois@llamasearch.ai
    
    /// Whether the account is suspended
    pub is_suspended: Option<bool>,
    
    /// Whether the account is NSFW
    pub over_18: Option<bool>,
    
    /// Account icon image URL
    pub icon_img: Option<String>,
    
    /// Moderated subreddits
    pub moderated_subreddits: Option<Vec<ModeratedSubreddit>>,
    
    /// Whether the current user has blocked this user
    pub is_blocked: Option<bool>,
    
    /// Whether this user has been blocked by the current user
    pub blocked: Option<bool>,
    
    /// Additional custom fields returned by Reddit
    #[serde(flatten)]
    pub additional_fields: HashMap<String, serde_json::Value>,
}

/// Information about a moderated subreddit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeratedSubreddit {
    /// Subreddit name
    pub name: String,
    
    /// Subreddit title
    pub title: Option<String>,
    
    /// Subscriber count
    pub subscribers: Option<i32>,
    
    /// Moderator permissions
    pub mod_permissions: Vec<String>,
}

/// A flair template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlairTemplate {
    /// Template ID
    pub id: String,
    
    /// Flair text
    pub text: String,
    
    /// Text color
    pub text_color: String,
    
    /// Background color
    pub background_color: String,
    
    /// CSS class
    pub css_class: Option<String>,
    
    /// Whether the flair is editable by users
    pub text_editable: bool,
    
    /// Maximum flair text length
    pub max_emojis: Option<i32>,
    
    /// Allowed user groups
    pub allowable_content: Option<String>,
    
    /// Flair type
    pub flair_type: Option<String>,
}

/// An award on a post or comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Award {
    /// Award ID
    pub id: String,
    
    /// Award name
    pub name: String,
    
    /// Award description
    pub description: Option<String>,
    
    /// Award icon URL
    pub icon_url: Option<String>,
    
    /// Award static icon URL
    pub static_icon_url: Option<String>,
    
    /// Award coin price
    pub coin_price: i32,
    
    /// Award count (how many were given)
    pub count: i32,
    
    /// Days of Reddit Premium granted by this award
    pub days_of_premium: Option<i32>,
    
    /// Coins granted to recipient
    pub coin_reward: Option<i32>,
    
    /// Award tier
    pub award_type: Option<String>,
    
    /// Subreddit ID, if a subreddit-specific award
    pub subreddit_id: Option<String>,
}

/// A ModAction from the mod log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModAction {
    /// Action ID
    pub id: String,
    
    /// Action type
    pub action: String,
    
    /// Moderator's username
    pub mod_username: String,
    
    /// Target fullname
    pub target_fullname: Option<String>,
    
    /// Target title/body
    pub target_title: Option<String>,
    
    /// Target author's username
    pub target_Author: Nik Jois
    
    /// Target permalink
    pub target_permalink: Option<String>,
    
    /// Subreddit name
    pub subreddit: String,
    
    /// Subreddit name prefixed
    pub subreddit_name_prefixed: String,
    
    /// Action details
    pub details: Option<String>,
    
    /// Description of the action
    pub description: Option<String>,
    
    /// Creation time (UTC)
    #[serde(with = "timestamp_seconds")]
    pub created_utc: DateTime<Utc>,
}

/// A private message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique ID of the message (without prefix)
    pub id: String,
    
    /// Fullname of the message (with t4_ prefix)
    pub name: String,
    
    /// Message subject
    pub subject: String,
    
    /// Message body
    pub body: String,
    
    /// HTML formatted body
    pub body_html: Option<String>,
    
    /// Sender's username
    pub Author: Nik Jois
    
    /// Recipient's username
    pub dest: String,
    
    /// Creation time (UTC)
    #[serde(with = "timestamp_seconds")]
    pub created_utc: DateTime<Utc>,
    
    /// Subreddit name, if a modmail
    pub subreddit: Option<String>,
    
    /// Context (for comment replies)
    pub context: Option<String>,
    
    /// Whether the message is unread
    pub new: bool,
    
    /// Whether the message was sent from inbox
    pub was_comment: bool,
    
    /// Parent ID, if part of a conversation
    pub parent_id: Option<String>,
    
    /// Additional custom fields returned by Reddit
    #[serde(flatten)]
    pub additional_fields: HashMap<String, serde_json::Value>,
}

/// Preview images data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preview {
    /// Preview images
    pub images: Vec<Image>,
    
    /// Whether the preview is enabled
    pub enabled: bool,
}

/// Image data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    /// Source image
    pub source: ImageSource,
    
    /// Resized variants
    pub resolutions: Vec<ImageSource>,
    
    /// Image variants
    pub variants: Option<HashMap<String, ImageVariant>>,
    
    /// Image ID
    pub id: Option<String>,
}

/// Image source data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    /// Image URL
    pub url: String,
    
    /// Image width
    pub width: i32,
    
    /// Image height
    pub height: i32,
}

/// Image variant (e.g., GIF/MP4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageVariant {
    /// Source of the variant
    pub source: ImageSource,
    
    /// Resolutions of the variant
    pub resolutions: Vec<ImageSource>,
}

/// Gallery data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryData {
    /// Gallery items
    pub items: Vec<GalleryItem>,
}

/// Gallery item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryItem {
    /// Media ID
    pub media_id: String,
    
    /// Item ID
    pub id: i32,
    
    /// Caption
    pub caption: Option<String>,
    
    /// Link URL
    pub outbound_url: Option<String>,
}

/// Data about whether a post/comment has been edited
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Edited {
    /// Not edited
    NotEdited(bool),
    
    /// Edited at timestamp
    Timestamp(f64),
}

impl Edited {
    /// Check if the item has been edited
    pub fn is_edited(&self) -> bool {
        match self {
            Edited::NotEdited(false) => false,
            _ => true,
        }
    }
    
    /// Get the edit timestamp, if available
    pub fn timestamp(&self) -> Option<DateTime<Utc>> {
        match self {
            Edited::Timestamp(ts) => Some(DateTime::from_timestamp(*ts as i64, 0).unwrap_or_else(|| Utc::now())),
            _ => None,
        }
    }
}

mod timestamp_seconds {
    use chrono::{DateTime, Utc, TimeZone};
    use serde::{self, Deserialize, Serializer, Deserializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(date.timestamp() as f64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = f64::deserialize(deserializer)?;
        let seconds_int = seconds as i64;
        let nanos = ((seconds - seconds_int as f64) * 1_000_000_000.0) as u32;
        
        Utc.timestamp_opt(seconds_int, nanos)
            .single()
            .ok_or_else(|| serde::de::Error::custom(format!("Invalid timestamp: {}", seconds)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_thing_kind_prefix() {
        assert_eq!(ThingKind::Comment.prefix(), "t1_");
        assert_eq!(ThingKind::Account.prefix(), "t2_");
        assert_eq!(ThingKind::Link.prefix(), "t3_");
        assert_eq!(ThingKind::Message.prefix(), "t4_");
        assert_eq!(ThingKind::Subreddit.prefix(), "t5_");
        assert_eq!(ThingKind::Award.prefix(), "t6_");
        assert_eq!(ThingKind::Listing.prefix(), "");
        assert_eq!(ThingKind::More.prefix(), "");
        assert_eq!(ThingKind::Unknown.prefix(), "");
    }
    
    #[test]
    fn test_thing_kind_from_fullname() {
        assert_eq!(ThingKind::from_fullname("t1_abc123"), Some(ThingKind::Comment));
        assert_eq!(ThingKind::from_fullname("t2_abc123"), Some(ThingKind::Account));
        assert_eq!(ThingKind::from_fullname("t3_abc123"), Some(ThingKind::Link));
        assert_eq!(ThingKind::from_fullname("t4_abc123"), Some(ThingKind::Message));
        assert_eq!(ThingKind::from_fullname("t5_abc123"), Some(ThingKind::Subreddit));
        assert_eq!(ThingKind::from_fullname("t6_abc123"), Some(ThingKind::Award));
        assert_eq!(ThingKind::from_fullname("invalid"), None);
    }
    
    #[test]
    fn test_thing_kind_matches_fullname() {
        assert!(ThingKind::Comment.matches_fullname("t1_abc123"));
        assert!(!ThingKind::Comment.matches_fullname("t2_abc123"));
        assert!(ThingKind::Link.matches_fullname("t3_abc123"));
        assert!(!ThingKind::Listing.matches_fullname("any_value"));
    }
    
    #[test]
    fn test_edited() {
        let not_edited = Edited::NotEdited(false);
        let edited_bool = Edited::NotEdited(true);
        let edited_time = Edited::Timestamp(1609459200.0); // 2021-01-01
        
        assert!(!not_edited.is_edited());
        assert!(edited_bool.is_edited());
        assert!(edited_time.is_edited());
        
        assert!(not_edited.timestamp().is_none());
        assert!(edited_bool.timestamp().is_none());
        assert!(edited_time.timestamp().is_some());
    }
} 