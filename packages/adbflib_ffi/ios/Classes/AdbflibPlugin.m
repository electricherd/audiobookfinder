#import "AdbflibPlugin.h"
#if __has_include(<adbflib/adbflib-Swift.h>)
#import <adbflib/adbflib-Swift.h>
#else
// Support project import fallback if the generated compatibility header
// is not copied when this plugin is created as a library.
// https://forums.swift.org/t/swift-static-libraries-dont-copy-generated-objective-c-header/19816
#import "adbflib-Swift.h"
#endif

@implementation AdbflibPlugin
+ (void)registerWithRegistrar:(NSObject<FlutterPluginRegistrar>*)registrar {
  [SwiftScrapPlugin registerWithRegistrar:registrar];
}
@end
