part of 'help_overview_bloc.dart';

sealed class HelpOverviewState extends Equatable {
  const HelpOverviewState();

  @override
  List<Object?> get props => [];
}

class HelpOverviewInitial extends HelpOverviewState {
  const HelpOverviewInitial();
}

class HelpOverviewLoadInProgress extends HelpOverviewState {
  const HelpOverviewLoadInProgress();
}

class HelpOverviewLoadSuccess extends HelpOverviewState {
  final List<HelpCategory> categories;

  const HelpOverviewLoadSuccess(this.categories);

  @override
  List<Object?> get props => [categories];
}

class HelpOverviewLoadFailure extends HelpOverviewState implements ErrorState {
  @override
  final ApplicationError error;

  const HelpOverviewLoadFailure(this.error);

  @override
  List<Object?> get props => [error];
}
