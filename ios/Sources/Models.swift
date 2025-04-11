import Foundation

// Base models that match the backend data structure

struct Account: Identifiable, Codable {
    let id: Int
    let name: String
    let balance: Decimal
    let createdAt: Date
    let updatedAt: Date
    
    enum CodingKeys: String, CodingKey {
        case id, name, balance
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}

struct Envelope: Identifiable, Codable {
    let id: Int
    let name: String
    let amount: Decimal
    let createdAt: Date
    let updatedAt: Date
    
    enum CodingKeys: String, CodingKey {
        case id, name, amount
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}

struct Goal: Identifiable, Codable {
    let id: Int
    let name: String
    let targetAmount: Decimal
    let accumulatedAmount: Decimal
    let targetDate: Date?
    let createdAt: Date
    let updatedAt: Date
    
    enum CodingKeys: String, CodingKey {
        case id, name
        case targetAmount = "target_amount"
        case accumulatedAmount = "accumulated_amount"
        case targetDate = "target_date"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
    
    var progress: Double {
        if targetAmount == 0 { return 0 }
        return  NSDecimalNumber(decimal: accumulatedAmount).doubleValue  / NSDecimalNumber(decimal: targetAmount).doubleValue
    }
    
    var isCompleted: Bool {
        return accumulatedAmount >= targetAmount
    }
}

// Request and response models for API interactions

struct CreateAccountRequest: Codable {
    let name: String
    let balance: Decimal
}

struct UpdateAccountRequest: Codable {
    let name: String
    let balance: Decimal
}

struct CreateEnvelopeRequest: Codable {
    let name: String
    let amount: Decimal
}

struct UpdateEnvelopeRequest: Codable {
    let name: String
    let amount: Decimal
}

struct CreateGoalRequest: Codable {
    let name: String
    let targetAmount: Decimal
    let targetDate: Date?
    
    enum CodingKeys: String, CodingKey {
        case name
        case targetAmount = "target_amount"
        case targetDate = "target_date"
    }
}

struct UpdateGoalRequest: Codable {
    let name: String
    let targetAmount: Decimal
    let accumulatedAmount: Decimal
    let targetDate: Date?
    
    enum CodingKeys: String, CodingKey {
        case name
        case targetAmount = "target_amount"
        case accumulatedAmount = "accumulated_amount"
        case targetDate = "target_date"
    }
}

// Extension to help with date formatting
extension DateFormatter {
    static let iso8601Full: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSSZZZZZ"
        formatter.calendar = Calendar(identifier: .iso8601)
        formatter.timeZone = TimeZone(secondsFromGMT: 0)
        formatter.locale = Locale(identifier: "en_US_POSIX")
        return formatter
    }()
}

// Extension to help with currency formatting
extension NumberFormatter {
    static let currencyFormatter: NumberFormatter = {
        let formatter = NumberFormatter()
        formatter.numberStyle = .currency
        formatter.minimumFractionDigits = 2
        formatter.maximumFractionDigits = 2
        return formatter
    }()
}
