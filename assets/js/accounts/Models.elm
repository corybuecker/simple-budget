module Accounts.Models exposing (Account, Adjustment, Model, emptyModel, newAccount, newAdjustment)

import Http


type alias Account =
    { id : Int
    , name : String
    , debt : Bool
    , balance : Float
    , adjustments : List Adjustment
    , adjustmentsVisible : Bool
    }


newAccount : Account
newAccount =
    Account 0 "" False 0.0 [] False


type alias Model =
    { accounts : List Account
    , activeAccount : Maybe Account
    , activeAdjustment : Maybe Adjustment
    , error : Maybe Http.Error
    , modalOpen : String
    }


emptyModel : Model
emptyModel =
    Model [] Nothing Nothing Nothing ""


newAdjustment : Adjustment
newAdjustment =
    Adjustment 0 0 "" 0.0


type alias Adjustment =
    { accountId : Int
    , id : Int
    , title : String
    , total : Float
    }
