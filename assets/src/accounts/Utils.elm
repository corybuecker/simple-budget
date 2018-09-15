module Accounts.Utils exposing (accountDecoder, accountUpdatedDecoder, accountsDecoder, adjustmentDecoder, balance, encode, id, name)

import Accounts.Models exposing (Account, Adjustment)
import Json.Decode
import Json.Decode.Pipeline
import Json.Encode as Encode exposing (Value, int, object, string)


accountsDecoder : Json.Decode.Decoder (List Account)
accountsDecoder =
    Json.Decode.field "data" (Json.Decode.list accountDecoder)


accountDecoder : Json.Decode.Decoder Account
accountDecoder =
    Json.Decode.succeed Account
        |> Json.Decode.Pipeline.required "id" Json.Decode.int
        |> Json.Decode.Pipeline.required "name" Json.Decode.string
        |> Json.Decode.Pipeline.required "debt" Json.Decode.bool
        |> Json.Decode.Pipeline.required "balance" Json.Decode.float
        |> Json.Decode.Pipeline.required "adjustments" (Json.Decode.list adjustmentDecoder)


accountUpdatedDecoder : Json.Decode.Decoder Account
accountUpdatedDecoder =
    Json.Decode.field "data" accountDecoder


adjustmentDecoder : Json.Decode.Decoder Adjustment
adjustmentDecoder =
    Json.Decode.succeed Adjustment
        |> Json.Decode.Pipeline.required "id" Json.Decode.int
        |> Json.Decode.Pipeline.required "title" Json.Decode.string
        |> Json.Decode.Pipeline.required "total" Json.Decode.float


id : Int -> ( String, Encode.Value )
id value =
    ( "id", Encode.int value )


name : String -> ( String, Encode.Value )
name value =
    ( "name", Encode.string value )


balance : Float -> ( String, Encode.Value )
balance value =
    ( "balance", Encode.float value )


encode : Account -> Encode.Value
encode schema =
    Encode.object [ ( "account", Encode.object [ id schema.id, name schema.name, balance schema.balance ] ) ]
