module Adjustments.Models exposing (Adjustment, newAdjustment)


newAdjustment : Adjustment
newAdjustment =
    Adjustment 0 0 "" 0.0


type alias Adjustment =
    { accountId : Int
    , id : Int
    , title : String
    , total : Float
    }
