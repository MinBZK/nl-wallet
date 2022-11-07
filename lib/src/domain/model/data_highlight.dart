import 'package:equatable/equatable.dart';

class DataHighlight extends Equatable {
  final String title;
  final String subtitle;
  final String? image;

  const DataHighlight({
    required this.title,
    required this.subtitle,
    required this.image,
  });

  @override
  List<Object?> get props => [title, subtitle, image];
}
