import 'package:wallet_core/core.dart';

import '../../../domain/model/update/version_state.dart';
import '../mapper.dart';

class FlutterVersionStateMapper extends Mapper<FlutterVersionState, VersionState> {
  FlutterVersionStateMapper();

  @override
  VersionState map(FlutterVersionState input) => input.map(
        ok: (state) => VersionStateOk(),
        notify: (state) => VersionStateNotify(),
        recommend: (state) => VersionStateRecommend(),
        warn: (state) => VersionStateWarn(timeUntilBlocked: Duration(seconds: state.expiresInSeconds.toInt())),
        block: (state) => VersionStateBlock(),
      );
}
