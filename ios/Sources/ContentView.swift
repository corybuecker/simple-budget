import SwiftUI
import GoogleSignIn
import GoogleSignInSwift

struct ContentView: View {
  @EnvironmentObject var userModel: UserModel
  @Environment(\.scenePhase) private var scenePhase
  
  @Sendable
  func loadData() async -> Void {
    guard let url = URL(string: "http://localhost:8000/authentication/token") else { return }

    var request = URLRequest(url: url)
    request.httpMethod = "POST"
    request.setValue("application/json", forHTTPHeaderField: "Content-Type")
    
    guard let encoded = try? JSONEncoder().encode(["id_token": userModel.idToken]) else {
      print("Failed to encode order")
      return
    }
      do {
        let (_, response) = try await URLSession.shared.upload(for: request, from: encoded)
        print("\(response)")
      } catch {
        print("Checkout failed: \(error.localizedDescription)")
      }
    
  }
  
  var body: some View {
    Text(userModel.idToken != nil ? "Logged in" : "Log in")
    
    if userModel.idToken != nil {
      VStack {
        Button("Load Data") {
          Task {
           await loadData()
          }
        }
        Button ("Sign out") {
          GIDSignIn.sharedInstance.signOut()
          userModel.idToken = nil
        }
      }
    } else {
      GoogleSignInButton(action: handleSignInButton(userModel))
    }

  }
}

func handleSignInButton(_ userModel: UserModel) -> () -> Void {
  return {
    let scene = UIApplication.shared.connectedScenes.first { $0.activationState == .foregroundActive }
    
    if let windowScene = scene as? UIWindowScene {
      guard let controller = windowScene.keyWindow?.rootViewController else { return }
      GIDSignIn.sharedInstance.signIn(withPresenting: controller)
    }
  }
}
