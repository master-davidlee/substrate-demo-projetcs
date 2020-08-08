#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet template with necessary imports


use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, ensure};
use frame_system::{self as system, ensure_signed};
use sp_std::prelude::*;
use frame_support::codec::{Encode,Decode};


#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, Default)]
pub struct Candidate{
	name:Vec<u8>,
	grades:u8,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, Default)]
pub struct VoteInfo<BlockNumber,AccountId>{
	end_at_block:BlockNumber,
	launcher:AccountId,
}
//type Vote<T> = Vote<<T as system::Trait>::BlockNumber>; 
#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, Default)]
pub struct WholeVote<BlockNumber,AccountId>{
	vote_info:VoteInfo<BlockNumber,AccountId>,
	candidates:Vec<Candidate>,

}

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	// Add other types and constants required to configure this pallet.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's stortage items.
decl_storage! {
	// It is importantt to update your storage name so that your pallet's
	// storage items are isolated from other pallets.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as Votetest {
		
		NewCandidate get(fn get_candidate): Candidate; //candidate and their grades
		
		Votes get(fn get_vote): map hasher(blake2_128_concat) Vec<u8>=> WholeVote<T::BlockNumber,T::AccountId>;//map vote's id to which vote

		AlreadyVote get(fn get_already_vote):map hasher(blake2_128_concat)  Vec<u8> => Vec<T::AccountId> ;//record already vote accountid
		
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		NewVote(Vec<u8>,AccountId),
		OneCandidatePush(Vec<u8>,Vec<u8>),
		VoteSuccess(Vec<u8>,Vec<u8>,AccountId),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		AlreayExistThisVote,
		TheVoteNotExist,
		NotTheVoteLauncher,
		NoSuchCandidate,
		YouHaveAlreadyVote,
		VoteAlreadyShutDown,
		
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		// this includes information about your errors in the node's metadata.
		// it is needed only if you are using errors in your pallet
		type Error = Error<T>;

		// Initializing events
		// this is needed only if you are using events in your pallet
		fn deposit_event() = default;

		/// Launch a vote
		#[weight = 10_000]
		pub fn launch_vote(origin, vote_id:Vec<u8>, end_after_nth_blocks: T::BlockNumber) -> dispatch::DispatchResult {
			//check signed or not
			let sender = ensure_signed(origin)?;
			let _sender = sender.clone();
			let current_block = <system::Module<T>>::block_number();
			let end_block_number = current_block+end_after_nth_blocks;
			let mut candidates:Vec<Candidate> =Vec::new();
			let vote_info = VoteInfo{
				
				end_at_block:end_block_number,
				launcher:sender,
			};
			let tmp_whole_vote =WholeVote{
				vote_info,
				candidates,
			};
			let _vote_id = vote_id.clone();
			ensure!(!Votes::<T>::contains_key(&vote_id), Error::<T>::AlreayExistThisVote);

			Votes::<T>::insert(vote_id,tmp_whole_vote);

			Self::deposit_event(RawEvent::NewVote(_vote_id,_sender));
			Ok(())
		}
		/// Another dummy entry point.
		/// takes no parameters, attempts to increment storage value, and possibly throws an error
		#[weight = 10_000]
		pub fn Insert_candidate(origin, vote_id:Vec<u8>, candidate_name:Vec<u8>) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Votes::<T>::contains_key(&vote_id), Error::<T>::TheVoteNotExist);
			
			let _candidate_name = candidate_name.clone();
			let _vote_id = vote_id.clone();
			let mut  new_candidate = Candidate{
				name:candidate_name,
				grades:0
			};
			let cur_votes = Votes::<T>::get(&vote_id);
			let owner = &cur_votes.vote_info.launcher;
			ensure!(sender == *owner, Error::<T>::NotTheVoteLauncher);
			let mut tmp_candidates =  cur_votes.candidates;
			tmp_candidates.push(new_candidate);
			let changeable_wholevote = WholeVote{
				candidates:tmp_candidates,
				..cur_votes
			};
			Votes::<T>::insert(_vote_id,changeable_wholevote);
			Self::deposit_event(RawEvent::OneCandidatePush(vote_id,_candidate_name));

			Ok(())
				
			
		}

		#[weight = 10_000]
		pub fn VoteFor(origin,vote_id:Vec<u8>,candidate_id:Vec<u8>) -> dispatch::DispatchResult{
			let sender = ensure_signed(origin)?;
			ensure!(Votes::<T>::contains_key(&vote_id), Error::<T>::TheVoteNotExist);
			ensure!(!<AlreadyVote<T>>::get(&vote_id).contains(&sender), Error::<T>::YouHaveAlreadyVote);
			let mut voter = sender.clone();
			let mut already_vote_guy = <AlreadyVote::<T>>::get(&vote_id);
			already_vote_guy.push(voter);
			<AlreadyVote::<T>>::insert(&vote_id,already_vote_guy);
			let _candidate_name = candidate_id.clone();
			let _vote_id = vote_id.clone();
			let cur_votes = Votes::<T>::get(&vote_id);//get current vote
			let tmp_vote_info = cur_votes.vote_info.clone();
			let curBlockNmuber = <system::Module<T>>::block_number();
			ensure!(curBlockNmuber<=tmp_vote_info.end_at_block, Error::<T>::VoteAlreadyShutDown);
			let mut tmp_candidates = cur_votes.candidates.clone();//
			let mut cur_grades = 0u8;
			let mut old_name :Vec<u8>= Vec::new();
			let mut count = 0usize;
			for exist_candidate in  cur_votes.candidates{
				if exist_candidate.name == candidate_id{
					cur_grades = exist_candidate.grades + 1;
					old_name = exist_candidate.name;
					break;
				}
				count += 1;
			}
			if cur_grades==0
			{	
				ensure!(!true,Error::<T>::NoSuchCandidate);
				}
			let change_candidate =Candidate{
				name: old_name,
				grades: cur_grades,
			};
			tmp_candidates.remove(count);
			tmp_candidates.push(change_candidate);
			let changeable_wholevote = WholeVote{
				candidates:tmp_candidates,
				..cur_votes
			};
			Votes::<T>::insert(_vote_id,changeable_wholevote);
			Self::deposit_event(RawEvent::VoteSuccess(vote_id,_candidate_name,sender));
			Ok(())
		}
	
	}
}
