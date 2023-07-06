import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/theme/wallet_theme.dart';

/// Widget that is to be used to wrap pumped test widgets.
/// Makes sure the theme, translations and MediaQuery is provided.
class WalletAppTestWidget extends StatelessWidget {
  final Widget child;
  final Brightness brightness;

  const WalletAppTestWidget({
    required this.child,
    this.brightness = Brightness.light,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      theme: brightness == Brightness.light ? WalletTheme.light : WalletTheme.dark,
      debugShowCheckedModeBanner: false,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: Material(child: child),
    );
  }
}

WidgetWrapper walletAppWrapper({Brightness brightness = Brightness.light, List<BlocProvider>? providers}) {
  return (child) {
    if (providers == null) return WalletAppTestWidget(brightness: brightness, child: child);
    return MultiBlocProvider(
      providers: providers,
      child: WalletAppTestWidget(brightness: brightness, child: child),
    );
  };
}

extension TestWidgetExtensions on Widget {
  /// Wraps the widget with a BlocProvider to provide the [bloc]
  /// and configures it to emit the provided [state].
  ///
  /// Useful to configure the state of a screen for a UI test.
  Widget withState<B extends BlocBase<S>, S>(B bloc, S state) => BlocProvider<B>(
        create: (c) {
          assert(bloc is MockBloc, 'Can only provide mocked state on MockBloc');
          whenListen(
            bloc,
            Stream<S>.empty(),
            initialState: state,
          );
          return bloc;
        },
        child: this,
      );
}
