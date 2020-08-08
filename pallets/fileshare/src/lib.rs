#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Decode, Encode};
/// A FRAME pallet template with necessary imports
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, Parameter,
};
use frame_system::{self as system, ensure_signed};
use sp_std::prelude::*;

#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, Default)]
pub struct File<AccountId> {
	name: Vec<u8>,
	description: Vec<u8>,
	uploader: AccountId,
	price: u8,
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
	trait Store for Module<T: Trait> as Fleshare {
		//store all files
		pub Files get(fn files): map hasher(blake2_128_concat) u32 => File<T::AccountId>;
		//store everyone's upload files
		pub EveryOneFiles get(fn get_someone_files): map hasher(blake2_128_concat) T::AccountId => Vec<u32>;
		//store everydownload files
		pub EveryOneDownFiles get(fn get_someone_download_files): map hasher(blake2_128_concat) T::AccountId => Vec<u32>;
		//store every file's ipfs
		pub EveryFileIpfs get(fn get_file_ipfs):map hasher(blake2_128_concat) u32 => Vec<u8>;
		//store who download the file
		pub WhoDownloadedFils get(fn get_downloader): map hasher(blake2_128_concat) u32=> Vec<T::AccountId>;
		//store file has been download how many times
		pub FileDownloadTimes get(fn get_file_download_times): map hasher(blake2_128_concat) u32=>u8;
		//every person's balance
		pub Balance get(fn get_balance): map hasher(blake2_128_concat) T::AccountId => u64;
		//Total supply
		pub Total get(fn get_total):u64= 1000000;
		//init status
		pub Init get(fn is_init):bool;
		//store all members
		pub Members get(fn get_members): Vec<T::AccountId>;
		//store sudo
		pub Sudo get(fn get_sudo):T::AccountId;
		//store fileid
		pub FileId get(fn now_id):u32=100000;

	}
}

// The pallet's events
decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		Initialized(AccountId),             //init the whole token
		Tranfer(AccountId, AccountId, u64), //from,to,value)
		AppendNewMember(AccountId),         //add new user to
		NewFileUpload(u32),                 //return new file id
		BuyResult(Vec<u8>),					//retrun the deal result
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		AlreadyInitialized,
		InsufficientFunds,
		OnlySudoCanAddNewMember,
		FileNotFound,
		LackOfBalance,
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

		//init the tokens
		#[weight = 10_000]
		pub fn init(origin) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(!Self::is_init(),Error::<T>::AlreadyInitialized);//ensure this function can only be run once
			Sudo::<T>::put(sender.clone());
			<Balance<T>>::insert(&sender,Self::get_total());//transfer all money to administrator
			Init::put(true);//close the function 
			Self::deposit_event(RawEvent::Initialized(sender));
			Ok(())

		}

		/// Transfer tokens from one accout to another
		#[weight = 10_000]
		pub fn transfer(origin, to:T::AccountId,value:u64) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
			let sender_balance = Self::get_balance(&sender);
			let receiver_balance = Self::get_balance(&to);//get buyer and receiver current balance

			ensure!(sender_balance>=value, Error::<T>::InsufficientFunds);
			let update_from_balance = sender_balance - value;
			let update_to_balance = receiver_balance + value;//calculate new balance

			Balance::<T>::insert(sender.clone(), update_from_balance);
			Balance::<T>::insert(to.clone(),     update_to_balance);//update everyone's balance
			Self::deposit_event(RawEvent::Tranfer(sender, to, value));
			Ok(())
		}

		//new user register
		#[weight =10_1000]
		pub fn register(origin, new_guy:T::AccountId)->dispatch::DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			ensure!(sender==Self::get_sudo(), Error::<T>::OnlySudoCanAddNewMember);//noly administrator can add new user
			<Members<T>>::append(&new_guy);//add new user to user list
			let _ = Self::transfer(origin,new_guy.clone(),1000);
			Self::deposit_event(RawEvent::AppendNewMember(new_guy));

			Ok(())
		}

		//upload files
		#[weight = 10_000]
		pub fn uploadfile(origin, name:Vec<u8>, description:Vec<u8>,ipfs_address:Vec<u8>,price:u8) ->dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
			let new_file=File{
				name:name,
				description:description,
				uploader:sender.clone(),
				price:price,
			};
			let file_id=Self::now_id()+1;//get file id ,id is the only id for a file
			let _file_id = file_id.clone();
			Files::<T>::insert(_file_id,new_file.clone());//add new file to  all files 
			let mut cur_files = Self::get_someone_files(&sender);
			cur_files.push(file_id.clone());
			EveryOneFiles::<T>::insert(&sender,cur_files);//update user's upload files
			EveryFileIpfs::insert(&file_id,ipfs_address);//update file's IPFS site
			let cur_downloaders :Vec<T::AccountId>=vec![];
			WhoDownloadedFils::<T>::insert(&file_id,cur_downloaders);//init file's downloader
			FileDownloadTimes::insert(&file_id,0);//init file's download times
			Self::deposit_event(RawEvent::NewFileUpload(file_id));
			Ok(())
		}
		//buy files
		#[weight =10_1000]
		pub fn buyfile(origin, file_id:u32) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			ensure!(Files::<T>::contains_key(&file_id), Error::<T>::FileNotFound );//enusre the file alreay exist
			let sender_balance = Self::get_balance(&sender);
			let tmp_file = Self::files(&file_id);
			ensure!(sender_balance >= tmp_file.price as u64 , Error::<T>::LackOfBalance);//ensure buyer has enough money
			let uploader = tmp_file.uploader;
			let _=Self::transfer(origin,uploader,tmp_file.price as u64);//transfer money
			let mut  cur_downloaders = Self::get_downloader(&file_id);
			cur_downloaders.push(sender.clone());
			WhoDownloadedFils::<T>::insert(&file_id,cur_downloaders);//update who download this file 
			let mut cur_downloadtime = Self::get_file_download_times(&file_id);
			cur_downloadtime += 1;
			FileDownloadTimes::insert(&file_id,cur_downloadtime);//update file downloaded times
			let	tmp_ipfs = Self::get_file_ipfs(&file_id);
			let mut cur_download_files = Self::get_someone_download_files(&sender);
			cur_download_files.push(file_id.clone());
			EveryOneDownFiles::<T>::insert(&sender, cur_download_files);//update buyer's downloaded files
			Self::deposit_event(RawEvent::BuyResult(tmp_ipfs));//return the file IPFS site 
			Ok(())

		}


	}
}
