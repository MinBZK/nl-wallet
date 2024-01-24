part of 'dashboard_bloc.dart';

abstract class DashboardEvent extends Equatable {
  const DashboardEvent();
}

class DashboardLoadTriggered extends DashboardEvent {
  final bool forceRefresh;

  const DashboardLoadTriggered({this.forceRefresh = false});

  @override
  List<Object?> get props => [forceRefresh];
}
