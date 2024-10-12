import SwiftUI
import GoogleSignIn
import GoogleSignInSwift

struct ContentView: View {
  @EnvironmentObject var userModel: UserModel
  var body: some View {
    Text(userModel.name ?? "User?")
    GoogleSignInButton(action: handleSignInButton)
  }
}

func handleSignInButton() {
  let scene = UIApplication.shared.connectedScenes.first { $0.activationState == .foregroundActive }
  
  if let windowScene = scene as? UIWindowScene {
    guard let controller = windowScene.keyWindow?.rootViewController else { return }
    GIDSignIn.sharedInstance.signIn(withPresenting: controller)
  }
}
