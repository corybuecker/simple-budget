module Accounts.Utils exposing (accountDecoder, accountUpdatedDecoder, accountsDecoder, adjustmentDecoder, adjustmentEncode, adjustmentId, adjustmentUpdatedDecoder, adjustmentsDecoder, balance, debt, encode, id, name, title, total)

import Accounts.Models
import Json.Decode
import Json.Decode.Pipeline
import Json.Encode


accountsDecoder : Json.Decode.Decoder (List Accounts.Models.Account)
accountsDecoder =
    Json.Decode.field "data" (Json.Decode.list accountDecoder)


accountDecoder : Json.Decode.Decoder Accounts.Models.Account
accountDecoder =
    Json.Decode.succeed Accounts.Models.Account
        |> Json.Decode.Pipeline.required "id" Json.Decode.int
        |> Json.Decode.Pipeline.required "name" Json.Decode.string
        |> Json.Decode.Pipeline.required "debt" Json.Decode.bool
        |> Json.Decode.Pipeline.required "balance" Json.Decode.float
        |> Json.Decode.Pipeline.required "adjustments" (Json.Decode.list adjustmentDecoder)
        |> Json.Decode.Pipeline.hardcoded False


accountUpdatedDecoder : Json.Decode.Decoder Accounts.Models.Account
accountUpdatedDecoder =
    Json.Decode.field "data" accountDecoder


adjustmentDecoder : Json.Decode.Decoder Accounts.Models.Adjustment
adjustmentDecoder =
    Json.Decode.succeed Accounts.Models.Adjustment
        |> Json.Decode.Pipeline.required "account_id" Json.Decode.int
        |> Json.Decode.Pipeline.required "id" Json.Decode.int
        |> Json.Decode.Pipeline.required "title" Json.Decode.string
        |> Json.Decode.Pipeline.required "total" Json.Decode.float


id : Int -> ( String, Json.Encode.Value )
id value =
    ( "id", Json.Encode.int value )


name : String -> ( String, Json.Encode.Value )
name value =
    ( "name", Json.Encode.string value )


balance : Float -> ( String, Json.Encode.Value )
balance value =
    ( "balance", Json.Encode.float value )


debt : Bool -> ( String, Json.Encode.Value )
debt value =
    ( "debt", Json.Encode.bool value )


encode : Accounts.Models.Account -> Json.Encode.Value
encode schema =
    Json.Encode.object [ ( "account", Json.Encode.object [ id schema.id, name schema.name, debt schema.debt, balance schema.balance ] ) ]


adjustmentsDecoder : Json.Decode.Decoder (List Accounts.Models.Adjustment)
adjustmentsDecoder =
    Json.Decode.field "data" (Json.Decode.list adjustmentDecoder)


adjustmentUpdatedDecoder : Json.Decode.Decoder Accounts.Models.Adjustment
adjustmentUpdatedDecoder =
    Json.Decode.field "data" adjustmentDecoder


adjustmentId : Int -> ( String, Json.Encode.Value )
adjustmentId value =
    ( "id", Json.Encode.int value )


title : String -> ( String, Json.Encode.Value )
title value =
    ( "title", Json.Encode.string value )


total : Float -> ( String, Json.Encode.Value )
total value =
    ( "total", Json.Encode.float value )


adjustmentEncode : Accounts.Models.Adjustment -> Json.Encode.Value
adjustmentEncode schema =
    Json.Encode.object [ ( "adjustment", Json.Encode.object [ adjustmentId schema.id, title schema.title, total schema.total ] ) ]
