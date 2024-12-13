import 'package:equatable/equatable.dart';

sealed class VersionState extends Equatable {
  @override
  List<Object?> get props => [];
}

class VersionStateOk extends VersionState {}

class VersionStateNotify extends VersionState {}

class VersionStateRecommend extends VersionState {}

class VersionStateWarn extends VersionState {
  final Duration timeUntilBlocked;

  VersionStateWarn({required this.timeUntilBlocked});

  @override
  List<Object?> get props => [timeUntilBlocked, ...super.props];
}

class VersionStateBlock extends VersionState {}
