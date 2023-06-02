import 'package:core_domain/core_domain.dart';

abstract class UpdateDigidAuthStatusUseCase {
  Future<void> invoke(DigidState state);
}
