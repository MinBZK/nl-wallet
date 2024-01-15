part of 'check_attributes_bloc.dart';

abstract class CheckAttributesEvent extends Equatable {
  const CheckAttributesEvent();
}

class CheckAttributesLoadTriggered extends CheckAttributesEvent {
  @override
  List<Object?> get props => [];
}
