import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../wallet_routes.dart';
import '../common/widget/fake_paging_animated_switcher.dart';
import '../common/widget/text_icon_button.dart';
import 'bloc/introduction_bloc.dart';
import 'page/introduction_page.dart';
import 'widget/introduction_progress_stepper.dart';

class IntroductionScreen extends StatelessWidget {
  const IntroductionScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      restorationId: 'introduction_scaffold',
      body: WillPopScope(
        onWillPop: () async {
          final bloc = context.read<IntroductionBloc>();
          if (bloc.state.canGoBack) {
            bloc.add(const IntroductionBackPressed());
          }
          return !bloc.state.canGoBack;
        },
        child: _buildPage(),
      ),
    );
  }

  Widget _buildPage() {
    return BlocBuilder<IntroductionBloc, IntroductionState>(
      builder: (context, state) {
        Widget? result;
        if (state is IntroductionAppDisclaimer) result = _buildAppDisclaimerPage(context, state);
        if (state is IntroductionAppIntroduction) result = _buildAppIntroductionPage(context, state);
        if (state is IntroductionAppBenefits) result = _buildAppBenefitsPage(context, state);
        if (state is IntroductionAppSecurity) result = _buildAppSecurityPage(context, state);
        if (result == null) throw UnsupportedError('Unknown state: $state');
        return FakePagingAnimatedSwitcher(animateBackwards: state.didGoBack, child: result);
      },
    );
  }

  Widget _buildAppDisclaimerPage(BuildContext context, IntroductionState state) {
    return IntroductionPage(
      key: ValueKey(state.currentStep),
      image: const AssetImage('assets/non-free/images/image_introduction_app_disclaimer.png'),
      title: AppLocalizations.of(context).introductionAppDisclaimerPageTitle,
      progressStepper: _buildProgressStepper(state),
      onNextPressed: () => _onNextPressed(context),
      secondaryCta: _buildLanguageSelectCta(context),
    );
  }

  Widget _buildAppIntroductionPage(BuildContext context, IntroductionState state) {
    return IntroductionPage(
      key: ValueKey(state.currentStep),
      image: const AssetImage('assets/non-free/images/image_introduction_app_introduction.png'),
      title: AppLocalizations.of(context).introductionAppIntroPageTitle,
      progressStepper: _buildProgressStepper(state),
      onNextPressed: () => _onNextPressed(context),
      onBackPressed: () => _onBackPressed(context),
    );
  }

  Widget _buildAppBenefitsPage(BuildContext context, IntroductionState state) {
    return IntroductionPage(
      key: ValueKey(state.currentStep),
      image: const AssetImage('assets/non-free/images/image_introduction_app_benefits.png'),
      title: AppLocalizations.of(context).introductionAppBenefitsPageTitle,
      progressStepper: _buildProgressStepper(state),
      onNextPressed: () => _onNextPressed(context),
      onBackPressed: () => _onBackPressed(context),
    );
  }

  Widget _buildAppSecurityPage(BuildContext context, IntroductionState state) {
    return IntroductionPage(
      key: ValueKey(state.currentStep),
      image: const AssetImage('assets/non-free/images/image_introduction_app_security.png'),
      title: AppLocalizations.of(context).introductionAppSecurityPageTitle,
      progressStepper: _buildProgressStepper(state),
      onNextPressed: () => Navigator.pushReplacementNamed(context, WalletRoutes.setupSecurityRoute),
      secondaryCta: _buildPrivacyPolicyCta(context),
      onBackPressed: () => _onBackPressed(context),
    );
  }

  Widget _buildProgressStepper(IntroductionState state) {
    return IntroductionProgressStepper(currentStep: state.currentStep, totalSteps: state.totalSteps);
  }

  Widget _buildLanguageSelectCta(BuildContext context) {
    return TextIconButton(
      icon: Icons.language,
      iconPosition: IconPosition.start,
      onPressed: () {},
      centerChild: false,
      child: Text(AppLocalizations.of(context).introductionLanguageSelectCta),
    );
  }

  Widget _buildPrivacyPolicyCta(BuildContext context) {
    return TextIconButton(
      icon: Icons.arrow_forward,
      onPressed: () {},
      child: Text(AppLocalizations.of(context).introductionPrivacyPolicyCta),
    );
  }

  void _onNextPressed(BuildContext context) {
    context.read<IntroductionBloc>().add(const IntroductionNextPressed());
  }

  void _onBackPressed(BuildContext context) {
    context.read<IntroductionBloc>().add(const IntroductionBackPressed());
  }
}
