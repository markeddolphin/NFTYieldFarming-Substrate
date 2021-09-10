/// The number of parents to search in get_sharing_proportions function
pub const GET_SHARING_PROPORTIONS_DEPTH: usize = 3;
/// The duration an `UpdateMediaProposal` will remain valid
pub const UPDATE_MEDIA_PROPOSAL_DURATION: u64 = contract_utils::time::WEEK;
/// The total number of shares a collab can have for a media
pub const COLLAB_SHARE_COUNT: u128 = 1_000_000_000;
