import 'package:wallet_mock/mock.dart' as core;

import '../../../domain/model/document.dart';
import '../mapper.dart';

class DocumentMapper extends Mapper<core.Document, Document> {
  DocumentMapper();

  @override
  Document map(core.Document input) => Document(
    title: input.title,
    fileName: input.fileName,
    url: input.url,
  );
}
