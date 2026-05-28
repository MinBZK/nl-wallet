part of 'help_overview_bloc.dart';

abstract class HelpOverviewEvent extends Equatable {
  const HelpOverviewEvent();

  @override
  List<Object?> get props => [];
}

class HelpOverviewLoadTriggered extends HelpOverviewEvent {
  const HelpOverviewLoadTriggered();
}
