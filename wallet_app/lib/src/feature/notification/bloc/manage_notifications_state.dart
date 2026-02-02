part of 'manage_notifications_bloc.dart';

sealed class ManageNotificationsState extends Equatable {
  const ManageNotificationsState();

  @override
  List<Object?> get props => [];
}

class ManageNotificationsInitial extends ManageNotificationsState {
  const ManageNotificationsInitial();
}

class ManageNotificationsError extends ManageNotificationsState {
  const ManageNotificationsError();
}

class ManageNotificationsLoaded extends ManageNotificationsState {
  final bool pushEnabled;

  const ManageNotificationsLoaded({required this.pushEnabled});

  ManageNotificationsLoaded toggled() => ManageNotificationsLoaded(pushEnabled: !pushEnabled);

  @override
  List<Object> get props => [pushEnabled];
}
