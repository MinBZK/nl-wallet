package nl.wallet.bluetooth

import android.app.Activity
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothManager
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.embedding.engine.plugins.activity.ActivityAware
import io.flutter.embedding.engine.plugins.activity.ActivityPluginBinding
import io.flutter.plugin.common.EventChannel
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import io.flutter.plugin.common.PluginRegistry

class BluetoothPlugin : FlutterPlugin, MethodChannel.MethodCallHandler, EventChannel.StreamHandler, ActivityAware,
    PluginRegistry.ActivityResultListener {

    private lateinit var channel: MethodChannel
    private lateinit var stateChannel: EventChannel
    private var context: Context? = null
    private var stateReceiver: BroadcastReceiver? = null
    private var activity: Activity? = null
    private var bluetoothAdapter: BluetoothAdapter? = null
    private var pendingResult: MethodChannel.Result? = null

    private val REQUEST_ENABLE_BT = 1001

    override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        channel = MethodChannel(binding.binaryMessenger, "bluetooth")
        channel.setMethodCallHandler(this)
        stateChannel = EventChannel(binding.binaryMessenger, "bluetooth/state")
        stateChannel.setStreamHandler(this)
        context = binding.applicationContext
        val mgr = binding.applicationContext.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
        bluetoothAdapter = mgr.adapter
    }

    override fun onMethodCall(call: MethodCall, result: MethodChannel.Result) {
        when (call.method) {
            "isBluetoothEnabled" -> result.success(bluetoothAdapter?.isEnabled == true)
            "enableBluetooth" -> {
                if (bluetoothAdapter?.isEnabled == true) {
                    result.success(true)
                    return
                }

                val intent = Intent(BluetoothAdapter.ACTION_REQUEST_ENABLE)
                if (activity != null) {
                    pendingResult = result
                    activity?.startActivityForResult(intent, REQUEST_ENABLE_BT)
                } else {
                    result.error("NO_ACTIVITY", "Activity is null", null)
                }
            }
            else -> result.notImplemented()
        }
    }

    override fun onListen(arguments: Any?, events: EventChannel.EventSink) {
        val receiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context, intent: Intent) {
                // Ignore the TURNING_ON/TURNING_OFF transitions, the adapter can't be used until they finish
                when (intent.getIntExtra(BluetoothAdapter.EXTRA_STATE, BluetoothAdapter.ERROR)) {
                    BluetoothAdapter.STATE_ON -> events.success(true)
                    BluetoothAdapter.STATE_OFF -> events.success(false)
                }
            }
        }
        // ACTION_STATE_CHANGED is a protected system broadcast, so no RECEIVER_(NOT_)EXPORTED flag is needed
        context?.registerReceiver(receiver, IntentFilter(BluetoothAdapter.ACTION_STATE_CHANGED))
        stateReceiver = receiver
        // Read the state only after registering, so a change in between is not missed
        events.success(bluetoothAdapter?.isEnabled == true)
    }

    override fun onCancel(arguments: Any?) {
        stateReceiver?.let { context?.unregisterReceiver(it) }
        stateReceiver = null
    }

    // Handle the result from the "Enable Bluetooth" dialog
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?): Boolean {
        if (requestCode == REQUEST_ENABLE_BT) {
            pendingResult?.success(resultCode == Activity.RESULT_OK)
            pendingResult = null
            return true
        }
        return false
    }

    // --- ActivityAware Implementation ---
    override fun onAttachedToActivity(binding: ActivityPluginBinding) {
        activity = binding.activity
        binding.addActivityResultListener(this)
    }

    override fun onDetachedFromActivity() {
        activity = null
    }

    override fun onReattachedToActivityForConfigChanges(binding: ActivityPluginBinding) {
        activity = binding.activity
        binding.addActivityResultListener(this)
    }

    override fun onDetachedFromActivityForConfigChanges() {
        activity = null
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        channel.setMethodCallHandler(null)
        stateChannel.setStreamHandler(null)
        onCancel(null)
        context = null
    }
}
