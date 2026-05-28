import 'package:equatable/equatable.dart';

class HelpTopic extends Equatable {
  final String id;
  final String title;

  const HelpTopic({
    required this.id,
    required this.title,
  });

  @override
  List<Object?> get props => [id, title];
}
