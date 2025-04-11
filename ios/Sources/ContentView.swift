import SwiftUI
import GoogleSignIn
import GoogleSignInSwift

struct ContentView: View {
  @EnvironmentObject var userModel: UserModel
  @EnvironmentObject var budgetViewModel: BudgetViewModel
  @Environment(\.scenePhase) private var scenePhase
  
  var body: some View {
    NavigationView {
      VStack {
        if userModel.idToken == nil {
          // Login view
          VStack(spacing: 20) {
            Text("Simple Budget")
              .font(.largeTitle)
              .fontWeight(.bold)
            
            Text("Please sign in to continue")
              .foregroundColor(.secondary)
            
            GoogleSignInButton(action: handleSignInButton(userModel))
              .padding(.top, 20)
          }
          .padding()
        } else {
          // Main app view with tabs
          TabView {
            // Accounts Tab
            AccountsView(viewModel: budgetViewModel)
              .tabItem {
                Label("Accounts", systemImage: "creditcard")
              }
            
            // Envelopes Tab
            EnvelopesView(viewModel: budgetViewModel)
              .tabItem {
                Label("Envelopes", systemImage: "folder")
              }
            
            // Goals Tab
            GoalsView(viewModel: budgetViewModel)
              .tabItem {
                Label("Goals", systemImage: "target")
              }
            
            // Settings Tab
            SettingsView(viewModel: budgetViewModel)
              .tabItem {
                Label("Settings", systemImage: "gear")
              }
          }
        }
        
        // Loading indicator
        if budgetViewModel.isLoading {
          ProgressView()
            .progressViewStyle(CircularProgressViewStyle())
            .scaleEffect(1.5)
            .padding()
        }
        
        // Error message
        if let error = budgetViewModel.error {
          Text(error)
            .foregroundColor(.red)
            .padding()
        }
      }
      .navigationBarTitle("", displayMode: .inline)
      .navigationBarItems(trailing: userModel.idToken != nil ? Button("Sign Out") {
        GIDSignIn.sharedInstance.signOut()
        userModel.idToken = nil
      } : nil)
    }
  }
}

// MARK: - Account Views

struct AccountsView: View {
  @ObservedObject var viewModel: BudgetViewModel
  @State private var showingAddAccount = false
  @State private var newAccountName = ""
  @State private var newAccountBalance = ""
  
  var body: some View {
    List {
      ForEach(viewModel.accounts) { account in
        HStack {
          VStack(alignment: .leading) {
            Text(account.name)
              .font(.headline)
            Text("Created: \(formattedDate(account.createdAt))")
              .font(.caption)
              .foregroundColor(.secondary)
          }
          
          Spacer()
          
          Text(formatCurrency(account.balance))
            .font(.headline)
        }
        .padding(.vertical, 8)
      }
    }
    .navigationTitle("Accounts")
    .toolbar {
      ToolbarItem(placement: .navigationBarTrailing) {
        Button(action: {
          showingAddAccount = true
        }) {
          Image(systemName: "plus")
        }
      }
    }
    .sheet(isPresented: $showingAddAccount) {
      NavigationView {
        Form {
          Section(header: Text("Account Details")) {
            TextField("Account Name", text: $newAccountName)
            TextField("Initial Balance", text: $newAccountBalance)
              .keyboardType(.decimalPad)
          }
        }
        .navigationTitle("New Account")
        .navigationBarItems(
          leading: Button("Cancel") {
            showingAddAccount = false
            newAccountName = ""
            newAccountBalance = ""
          },
          trailing: Button("Save") {
            if let balance = Decimal(string: newAccountBalance) {
              viewModel.createAccount(name: newAccountName, balance: balance)
              showingAddAccount = false
              newAccountName = ""
              newAccountBalance = ""
            }
          }
          .disabled(newAccountName.isEmpty || newAccountBalance.isEmpty)
        )
      }
    }
    .onAppear {
      viewModel.loadAccounts()
    }
  }
}

// MARK: - Envelopes View

struct EnvelopesView: View {
  @ObservedObject var viewModel: BudgetViewModel
  @State private var showingAddEnvelope = false
  @State private var newEnvelopeName = ""
  @State private var newEnvelopeAmount = ""
  
  var body: some View {
    List {
      ForEach(viewModel.envelopes) { envelope in
        HStack {
          VStack(alignment: .leading) {
            Text(envelope.name)
              .font(.headline)
            Text("Created: \(formattedDate(envelope.createdAt))")
              .font(.caption)
              .foregroundColor(.secondary)
          }
          
          Spacer()
          
          Text(formatCurrency(envelope.amount))
            .font(.headline)
        }
        .padding(.vertical, 8)
      }
    }
    .navigationTitle("Envelopes")
    .toolbar {
      ToolbarItem(placement: .navigationBarTrailing) {
        Button(action: {
          showingAddEnvelope = true
        }) {
          Image(systemName: "plus")
        }
      }
    }
    .sheet(isPresented: $showingAddEnvelope) {
      NavigationView {
        Form {
          Section(header: Text("Envelope Details")) {
            TextField("Envelope Name", text: $newEnvelopeName)
            TextField("Amount", text: $newEnvelopeAmount)
              .keyboardType(.decimalPad)
          }
        }
        .navigationTitle("New Envelope")
        .navigationBarItems(
          leading: Button("Cancel") {
            showingAddEnvelope = false
            newEnvelopeName = ""
            newEnvelopeAmount = ""
          },
          trailing: Button("Save") {
            if let amount = Decimal(string: newEnvelopeAmount) {
              viewModel.createEnvelope(name: newEnvelopeName, amount: amount)
              showingAddEnvelope = false
              newEnvelopeName = ""
              newEnvelopeAmount = ""
            }
          }
          .disabled(newEnvelopeName.isEmpty || newEnvelopeAmount.isEmpty)
        )
      }
    }
    .onAppear {
      viewModel.loadEnvelopes()
    }
  }
}

// MARK: - Goals View

struct GoalsView: View {
  @ObservedObject var viewModel: BudgetViewModel
  @State private var showingAddGoal = false
  @State private var newGoalName = ""
  @State private var newGoalTargetAmount = ""
  @State private var newGoalTargetDate = Date()
  @State private var includeTargetDate = false
  
  var body: some View {
    List {
      ForEach(viewModel.goals) { goal in
        VStack(alignment: .leading) {
          HStack {
            Text(goal.name)
              .font(.headline)
            
            Spacer()
            
            Text(formatCurrency(goal.accumulatedAmount))
              .font(.headline)
          }
          
          Text("Target: \(formatCurrency(goal.targetAmount))")
            .font(.subheadline)
          
          if let targetDate = goal.targetDate {
            Text("Target Date: \(formattedDate(targetDate))")
              .font(.caption)
              .foregroundColor(.secondary)
          }
          
          ProgressView(value: Double(goal.progress))
            .progressViewStyle(LinearProgressViewStyle())
            .padding(.top, 4)
        }
        .padding(.vertical, 8)
      }
    }
    .navigationTitle("Goals")
    .toolbar {
      ToolbarItem(placement: .navigationBarTrailing) {
        Button(action: {
          showingAddGoal = true
        }) {
          Image(systemName: "plus")
        }
      }
    }
    .sheet(isPresented: $showingAddGoal) {
      NavigationView {
        Form {
          Section(header: Text("Goal Details")) {
            TextField("Goal Name", text: $newGoalName)
            TextField("Target Amount", text: $newGoalTargetAmount)
              .keyboardType(.decimalPad)
            
            Toggle("Set Target Date", isOn: $includeTargetDate)
            
            if includeTargetDate {
              DatePicker("Target Date", selection: $newGoalTargetDate, displayedComponents: .date)
            }
          }
        }
        .navigationTitle("New Goal")
        .navigationBarItems(
          leading: Button("Cancel") {
            showingAddGoal = false
            resetGoalForm()
          },
          trailing: Button("Save") {
            if let targetAmount = Decimal(string: newGoalTargetAmount) {
              viewModel.createGoal(
                name: newGoalName,
                targetAmount: targetAmount,
                targetDate: includeTargetDate ? newGoalTargetDate : nil
              )
              showingAddGoal = false
              resetGoalForm()
            }
          }
          .disabled(newGoalName.isEmpty || newGoalTargetAmount.isEmpty)
        )
      }
    }
    .onAppear {
      viewModel.loadGoals()
    }
  }
  
  private func resetGoalForm() {
    newGoalName = ""
    newGoalTargetAmount = ""
    newGoalTargetDate = Date()
    includeTargetDate = false
  }
}

// MARK: - Settings View

struct SettingsView: View {
  @ObservedObject var viewModel: BudgetViewModel
  @EnvironmentObject var userModel: UserModel
  
  var body: some View {
    Form {
      Section(header: Text("Account")) {
        HStack {
          Text("Status")
          Spacer()
          Text(userModel.idToken != nil ? "Signed In" : "Signed Out")
            .foregroundColor(.secondary)
        }
        
        Button("Sign Out") {
          GIDSignIn.sharedInstance.signOut()
          userModel.idToken = nil
        }
        .foregroundColor(.red)
      }
      
      Section(header: Text("Data")) {
        Button("Refresh All Data") {
          viewModel.loadAllData()
        }
      }
      
      Section(header: Text("About")) {
        HStack {
          Text("Version")
          Spacer()
          Text("1.0.4")
            .foregroundColor(.secondary)
        }
      }
    }
    .navigationTitle("Settings")
  }
}

// MARK: - Helper Functions

func handleSignInButton(_ userModel: UserModel) -> () -> Void {
  return {
    let scene = UIApplication.shared.connectedScenes.first { $0.activationState == .foregroundActive }
    
    if let windowScene = scene as? UIWindowScene {
      guard let controller = windowScene.keyWindow?.rootViewController else { return }
        GIDSignIn.sharedInstance.signIn(withPresenting: controller) { signInResult, error in
            guard error == nil else { return }
            guard let signInResult = signInResult else { return }
            userModel.idToken = signInResult.user.idToken?.tokenString
        }
    }
  }
}

func formattedDate(_ date: Date) -> String {
  let formatter = DateFormatter()
  formatter.dateStyle = .medium
  formatter.timeStyle = .none
  return formatter.string(from: date)
}

func formatCurrency(_ amount: Decimal) -> String {
  return NumberFormatter.currencyFormatter.string(from: NSDecimalNumber(decimal: amount)) ?? "$0.00"
}
