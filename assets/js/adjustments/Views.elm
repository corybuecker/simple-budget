module Adjustments.Views exposing (editView, renderAdjustment, renderAdjustments)

import Adjustments.Messages exposing (..)
import Adjustments.Models exposing (Adjustment)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Model exposing (Msg(..))


editView : Adjustment -> Html Adjustments.Messages.Msg
editView model =
    div []
        [ input [ type_ "text", value model.title, onInput Adjustments.Messages.TitleUpdated ] []
        , input [ type_ "text", value (String.fromFloat model.amount), onInput Adjustments.Messages.AmountUpdated ] []
        , button [ onClick Adjustments.Messages.SaveAdjustment ] [ text "Save" ]
        , button [ onClick Adjustments.Messages.DeleteAdjustment ] [ text "Delete" ]
        ]


renderAdjustments : List Adjustment -> Html Model.Msg
renderAdjustments adjustments =
    div []
        [ button [ onClick CreateAdjustment ] [ text "New Adjustment" ]
        , table
            []
            [ thead []
                [ tr []
                    [ th [] [ text "Title" ]
                    , th [] [ text "Amount" ]
                    ]
                ]
            , tbody [] (List.map renderAdjustment adjustments)
            ]
        ]


renderAdjustment : Adjustment -> Html Model.Msg
renderAdjustment adjustment =
    tr []
        [ td [ onClick (OpenAdjustmentEditor adjustment) ] [ text adjustment.title ]
        , td [] [ text (String.fromFloat adjustment.amount) ]
        ]
