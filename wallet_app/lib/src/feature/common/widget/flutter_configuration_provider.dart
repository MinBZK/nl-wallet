import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../bridge_generated.dart';
import '../../../wallet_core/typed_wallet_core.dart';
import 'centered_loading_indicator.dart';

class FlutterConfigurationProvider extends StatelessWidget {
  final Widget Function(FlutterConfiguration) builder;
  final FlutterConfiguration? defaultConfig;

  const FlutterConfigurationProvider({
    required this.builder,
    this.defaultConfig,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return StreamBuilder<FlutterConfiguration>(
      stream: context.read<TypedWalletCore>().observeConfig(),
      builder: (context, snapshot) {
        if (!snapshot.hasData && defaultConfig == null) return const CenteredLoadingIndicator();
        Fimber.i('Providing config: ${snapshot.data ?? defaultConfig}');
        return builder(snapshot.data ?? defaultConfig!);
      },
    );
  }
}
