import SwiftUI
import GoogleSignIn

class UserModel: ObservableObject {
  @Published var idToken: String?
}

@main
struct SimpleBudget: App {
  @StateObject var userModel: UserModel = .init()
  @StateObject var budgetViewModel: BudgetViewModel
  
  init() {
    let userModel = UserModel()
    self._userModel = StateObject(wrappedValue: userModel)
    self._budgetViewModel = StateObject(wrappedValue: BudgetViewModel(userModel: userModel))
  }
  
  var body: some Scene {
    WindowGroup {
      ContentView()
        .onOpenURL(perform: { (url: URL) in
          GIDSignIn.sharedInstance.handle(url)
        })
        .onAppear(perform: {
          GIDSignIn.sharedInstance.restorePreviousSignIn { user, error in
            guard error == nil else { return }
            userModel.idToken = user?.idToken?.tokenString
          }
        })
        .environmentObject(userModel)
        .environmentObject(budgetViewModel)
    }
  }
}
