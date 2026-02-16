import 'package:fimber/fimber.dart';

import '../../../domain/model/configuration/maintenance_window.dart';
import '../mapper.dart';

class MaintenanceWindowMapper extends Mapper<(String, String)?, MaintenanceWindow?> {
  MaintenanceWindowMapper();

  @override
  MaintenanceWindow? map((String, String)? input) {
    // Check for absent maintenance window data
    if (input == null) return null;

    try {
      // Parse RFC 3339 (ISO 8601) datetime strings to DateTime objects
      final startDateTime = DateTime.parse(input.$1);
      final endDateTime = DateTime.parse(input.$2);

      // Validate and emit maintenance window
      final window = MaintenanceWindow(
        startDateTime: startDateTime,
        endDateTime: endDateTime,
      );

      if (window.isValid) {
        return window;
      } else {
        Fimber.e('Invalid maintenance window; start is not before end: $input');
      }
    } catch (ex) {
      Fimber.e('Failed to parse maintenance window dates: $input', ex: ex);
    }

    // Fallback
    return null;
  }
}
