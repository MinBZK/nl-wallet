part of 'manage_notifications_bloc.dart';

abstract class ManageNotificationsEvent extends Equatable {
  const ManageNotificationsEvent();

  @override
  List<Object?> get props => [];
}

/// Request BLoC to refresh data
class ManageNotificationsLoadTriggered extends ManageNotificationsEvent {
  const ManageNotificationsLoadTriggered();
}

/// Notify BLoC about user toggling the push settings switch
class ManageNotificationsPushNotificationsToggled extends ManageNotificationsEvent {
  const ManageNotificationsPushNotificationsToggled();
}
