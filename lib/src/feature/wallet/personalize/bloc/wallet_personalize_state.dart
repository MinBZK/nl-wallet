part of 'wallet_personalize_bloc.dart';

const _kNrOfPages = 8;

abstract class WalletPersonalizeState extends Equatable {
  double get stepperProgress => 0.0;

  bool get canGoBack => false;

  bool get didGoBack => false;

  const WalletPersonalizeState();

  @override
  List<Object?> get props => [stepperProgress, didGoBack, canGoBack];
}

class WalletPersonalizeInitial extends WalletPersonalizeState {
  @override
  double get stepperProgress => 1 / _kNrOfPages;
}

class WalletPersonalizeLoadingPid extends WalletPersonalizeState {
  @override
  double get stepperProgress => 2 / _kNrOfPages;
}

class WalletPersonalizeCheckData extends WalletPersonalizeState {
  final String firstNames;
  final List<DataAttribute> availableAttributes;

  const WalletPersonalizeCheckData({required this.firstNames, required this.availableAttributes});

  @override
  double get stepperProgress => 3 / _kNrOfPages;

  @override
  List<Object?> get props => [firstNames, availableAttributes, ...super.props];
}

class WalletPersonalizeScanIdIntro extends WalletPersonalizeState {
  final bool afterBackPressed;

  const WalletPersonalizeScanIdIntro({this.afterBackPressed = false});

  @override
  double get stepperProgress => 4 / _kNrOfPages;

  @override
  bool get didGoBack => afterBackPressed;
}

class WalletPersonalizeScanId extends WalletPersonalizeState {
  @override
  double get stepperProgress => 5 / _kNrOfPages;

  @override
  bool get canGoBack => true;
}

class WalletPersonalizeLoadingPhoto extends WalletPersonalizeState {
  final Duration mockedScanDuration;

  const WalletPersonalizeLoadingPhoto(this.mockedScanDuration);

  @override
  double get stepperProgress => 6 / _kNrOfPages;
}

class WalletPersonalizePhotoAdded extends WalletPersonalizeState {
  final DataAttribute photo;

  const WalletPersonalizePhotoAdded(this.photo);

  @override
  double get stepperProgress => 7 / _kNrOfPages;
}

class WalletPersonalizeSuccess extends WalletPersonalizeState {
  final WalletCard pidCard;
  final Organization organization;

  const WalletPersonalizeSuccess(this.pidCard, this.organization);

  CardFront get cardFront => pidCard.front;

  @override
  double get stepperProgress => 1;

  @override
  List<Object?> get props => [pidCard, organization, ...super.props];
}

class WalletPersonalizeFailure extends WalletPersonalizeState {
  @override
  double get stepperProgress => 0;
}
