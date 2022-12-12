import 'package:equatable/equatable.dart';

class Document extends Equatable {
  final String title;
  final String fileName;
  final String url;

  const Document({
    required this.title,
    required this.fileName,
    required this.url,
  });

  @override
  List<Object?> get props => [title, fileName, url];
}
