module Savings.Models exposing (Model, Saving, emptyModel, newSaving)

import Http


newSaving : Saving
newSaving =
    Saving 0 "" 0.0


type alias Saving =
    { id : Int
    , title : String
    , amount : Float
    }


type alias Model =
    { savings : List Saving
    , activeSaving : Maybe Saving
    , error : Maybe Http.Error
    , modalOpen : String
    }


emptyModel : Model
emptyModel =
    Model [] Nothing Nothing ""
