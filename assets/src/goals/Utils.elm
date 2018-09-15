module Goals.Utils exposing (encode, end_date, goalDecoder, goalUpdatedDecoder, goalsDecoder, id, start_date, target, title)

import Goals.Models exposing (Goal)
import Json.Decode
import Json.Decode.Pipeline
import Json.Encode as Encode exposing (Value, int, object, string)


goalsDecoder : Json.Decode.Decoder (List Goal)
goalsDecoder =
    Json.Decode.field "data" (Json.Decode.list goalDecoder)


goalDecoder : Json.Decode.Decoder Goal
goalDecoder =
    Json.Decode.succeed Goal
        |> Json.Decode.Pipeline.required "id" Json.Decode.int
        |> Json.Decode.Pipeline.required "title" Json.Decode.string
        |> Json.Decode.Pipeline.required "start_date" Json.Decode.string
        |> Json.Decode.Pipeline.required "end_date" Json.Decode.string
        |> Json.Decode.Pipeline.required "target" Json.Decode.float


goalUpdatedDecoder : Json.Decode.Decoder Goal
goalUpdatedDecoder =
    Json.Decode.field "data" goalDecoder


id : Int -> ( String, Encode.Value )
id value =
    ( "id", Encode.int value )


title : String -> ( String, Encode.Value )
title value =
    ( "title", Encode.string value )


end_date : String -> ( String, Encode.Value )
end_date value =
    ( "end_date", Encode.string value )


start_date : String -> ( String, Encode.Value )
start_date value =
    ( "start_date", Encode.string value )


target : Float -> ( String, Encode.Value )
target value =
    ( "target", Encode.float value )


encode : Goal -> Encode.Value
encode schema =
    Encode.object [ ( "goal", Encode.object [ id schema.id, title schema.title, start_date schema.startDate, end_date schema.endDate, target schema.target ] ) ]
