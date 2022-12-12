import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:nfc_manager/nfc_manager.dart';

const _kScanIllustration = 'assets/non-free/images/scan_passport_illustration_2.png';

class WalletPersonalizeScanIdPage extends StatefulWidget {
  final VoidCallback onNfcDetected;

  const WalletPersonalizeScanIdPage({required this.onNfcDetected, Key? key}) : super(key: key);

  @override
  State<WalletPersonalizeScanIdPage> createState() => _WalletPersonalizeScanIdPageState();
}

class _WalletPersonalizeScanIdPageState extends State<WalletPersonalizeScanIdPage> {
  @override
  void initState() {
    super.initState();
    _startNfcScanner();
  }

  void _startNfcScanner() async {
    final isAvailable = await NfcManager.instance.isAvailable();
    if (!isAvailable) return;
    NfcManager.instance.startSession(
      onDiscovered: (NfcTag tag) async {
        widget.onNfcDetected();
        NfcManager.instance.stopSession();
      },
    );
  }

  @override
  void dispose() {
    NfcManager.instance.stopSession();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 24, horizontal: 16),
      child: Column(
        mainAxisSize: MainAxisSize.max,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            locale.walletPersonalizeScanIdPageTitle,
            style: Theme.of(context).textTheme.headline2,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          Text(
            locale.walletPersonalizeScanIdPageDescription,
            style: Theme.of(context).textTheme.bodyText1,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 32),
          Image.asset(
            _kScanIllustration,
            width: double.infinity,
            fit: BoxFit.cover,
          ),
          const Spacer(),
          if (kDebugMode) ElevatedButton(onPressed: widget.onNfcDetected, child: const Text('FAKE SCAN (DEV ONLY)'))
        ],
      ),
    );
  }
}
