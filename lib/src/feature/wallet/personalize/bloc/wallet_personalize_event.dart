part of 'wallet_personalize_bloc.dart';

abstract class WalletPersonalizeEvent extends Equatable {
  const WalletPersonalizeEvent();

  @override
  List<Object?> get props => [];
}

class WalletPersonalizeLoginWithDigidClicked extends WalletPersonalizeEvent {}

class WalletPersonalizeLoginWithDigidSucceeded extends WalletPersonalizeEvent {}

class WalletPersonalizeOfferingVerified extends WalletPersonalizeEvent {}

class WalletPersonalizeScanInitiated extends WalletPersonalizeEvent {}

class WalletPersonalizeScanEvent extends WalletPersonalizeEvent {}

class WalletPersonalizePhotoApproved extends WalletPersonalizeEvent {}

class WalletPersonalizeOnRetryClicked extends WalletPersonalizeEvent {}

class WalletPersonalizeOnBackPressed extends WalletPersonalizeEvent {}
