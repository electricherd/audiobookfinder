import 'dart:async';
import 'dart:ffi';
import 'package:ffi/ffi.dart';
import 'package:isolate/ports.dart';

import 'ffi.dart' as native;

class Adbflib {
  static setup() {
    native.store_dart_post_cobject(NativeApi.postCObject);
    print("Adbflib Setup Done");
  }

  Future<int> fileCountGood(String path) {
    var pathPointer = Utf8.toUtf8(path);
    final completer = Completer<int>();
    final sendPort = singleCompletePort(completer);
    final res = native.file_count_good(
      sendPort.nativePort,
      pathPointer,
    );
    if (res != 1) {
      _throwError();
    }
    return completer.future;
  }

  Future<int> findNewPeer() {
    // Dart integer seems to be of i64
    final completer = Completer<int>();
    final sendPort = singleCompletePort(completer);
    final res = native.find_new_peer(
      sendPort.nativePort
    );
    if (res != 1) {
      _throwError();
    }
    return completer.future;
  }

  int getOwnPeerId() {
    return native.get_own_peer();
  }

  Future<String> getNetUiMessages() {
    final completer = Completer<String>();
    final sendPort = singleCompletePort(completer);
    final res = native.get_net_ui_messages(
        sendPort.nativePort
    );
    if (res != 1) {
      _throwError();
    }
    return completer.future;
  }

  void _throwError() {
    final length = native.last_error_length();
    final Pointer<Utf8> message = allocate(count: length);
    native.error_message_utf8(message, length);
    final error = Utf8.fromUtf8(message);
    print(error);
    throw error;
  }
}
